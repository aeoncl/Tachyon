use std::str::FromStr;
use std::time::Duration;
use crate::matrix;
use crate::matrix::sync::{build_sliding_sync, sync};
use crate::tachyon::client_store::ClientStoreFacade;
use crate::notification::handlers::adl_handler::handle_adl;
use crate::notification::handlers::chg_handler::handle_chg;
use crate::notification::handlers::png_handler::handle_png;
use crate::notification::handlers::put_handler::handle_put;
use crate::notification::handlers::rml_handler::handle_rml;
use crate::notification::handlers::usr_handler::handle_usr;
use crate::notification::handlers::uum_handler::handle_uum;
use crate::notification::handlers::uux_handler::handle_uux;
use crate::notification::handlers::url_handler::handle_url;

use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::identifiers::MatrixIdCompatible;
use anyhow::anyhow;
use chrono::format;
use log::warn;
use matrix_sdk::notification_settings::IsOneToOne::No;
use matrix_sdk_ui::sync_service::SyncService;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrServer};
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::{NotServer, NotificationPayload, NotificationPayloadType};
use msnp::msnp::notification::command::url::UrlClient;
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::font_color::FontColor;
use msnp::shared::models::oim::{InboxMetadata, MailData, MailDataMessage};
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::notification::handlers::fqy_handler::handle_fqy;
use crate::notification::handlers::prp_handler::handle_prp;
use crate::notification::handlers::xfr_handler::handle_xfr;
use crate::tachyon::tachyon_client::TachyonClient;

pub(crate) async fn handle_command(command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, client_store: &ClientStoreFacade, local_client_data: &mut LocalClientData) -> Result<(), anyhow::Error> {

    let _command_result = match &local_client_data.phase {
        ConnectionPhase::Negotiating => {
            handle_negotiation(command, command_sender, local_client_data).await
        },
        ConnectionPhase::Authenticating  => {
            handle_auth(command, command_sender, &client_store, local_client_data).await
        },
        ConnectionPhase::Ready => {
            let client_data = local_client_data.client_data.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
            handle_ready(command, command_sender, client_data, local_client_data).await
        }
    };
    
    Ok(())

}

pub(crate) async fn handle_negotiation(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, local_client_data: &mut LocalClientData) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::VER(command) => {
            if command.first_candidate != MSNP18 && command.second_candidate != MSNP18 {
                //Unsupported protocol version
                //TODO add error code
                notif_sender.send(NotificationServerCommand::OUT).await?;
                return Ok(());
            }

            notif_sender.send(NotificationServerCommand::VER(command.get_response_for(MSNP18))).await?;
            Ok(())
        },
        NotificationClientCommand::CVR(command) => {
            local_client_data.phase = ConnectionPhase::Authenticating;
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
            Ok(())
        },
        _ => {
            Err(anyhow!("WTF are you doing here"))
        }
    }
}


const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"1000\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"100\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";
pub(crate) async fn handle_auth(command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, client_store: &ClientStoreFacade, local_store: &mut LocalClientData) -> Result<(), anyhow::Error> {
    match command {
        NotificationClientCommand::USR(command) => {
            match command.auth_type {
                AuthOperationTypeClient::Sso(content) => {
                    match content {
                        SsoPhaseClient::I { email_addr } => {
                            local_store.email_addr = email_addr;
                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU".to_string() }));
                            let gcf_response = RawCommand::with_payload("GCF 0", SHIELDS_PAYLOAD.as_bytes().to_vec());

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.send(NotificationServerCommand::RAW(gcf_response)).await?;
                        },

                        SsoPhaseClient::S { ticket_token, challenge: _, endpoint_guid } => {

                            let user_id = local_store.email_addr.to_owned_user_id();

                            let matrix_client = matrix::login::login_with_token(user_id.clone(), ticket_token.clone(), true).await?;
                            let sliding_sync = build_sliding_sync(&matrix_client).await?;

                            let endpoint_id = EndpointId::new(local_store.email_addr.clone(), Some(endpoint_guid));
                            let msn_user = MsnUser::new(endpoint_id);


                            let client_data = TachyonClient::new(msn_user.clone(), ticket_token.clone(), notif_sender.clone(), matrix_client.clone(), sliding_sync);
                            client_store.insert_client(ticket_token.as_str().to_owned(), client_data.clone());

                            local_store.token = ticket_token.clone();
                            local_store.client_data = Some(client_data.clone());
                            local_store.phase = ConnectionPhase::Ready;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: local_store.email_addr.clone(),
                                verified: true,
                                unknown_arg: false,
                            });

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;



                            notif_sender.send(NotificationServerCommand::RAW(RawCommand::without_payload("SBS 0 null"))).await?;

                            //TODO initial sync only and synchronous before anything else;

                            let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: DisplayName::new_from_ref("Hotmail"),
                                payload: MsgPayload::Raw(RawMsgPayloadFactory::get_msmsgs_profile(
                                    &msn_user.uuid.get_puid(),
                                    msn_user.get_email_address(),
                                    &ticket_token,
                                )),
                            });

                            notif_sender.send(initial_profile_msg).await?;

                            //Todo fetch endpoint data
                            let endpoint_data = b"<Data></Data>";
                            notif_sender
                                .send(NotificationServerCommand::RAW(RawCommand::with_payload(
                                    &format!("UBX 1:{}", &msn_user.get_email_address().as_str()),
                                    endpoint_data.to_vec(),
                                )))
                                .await?;

                            sync(client_data, local_store.client_kill_snd.clone(), local_store.client_kill_recv.resubscribe());

                            let initial_mail_data = NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: DisplayName::new_from_ref("Hotmail"),
                                payload: MsgPayload::Raw(RawMsgPayloadFactory::get_initial_mail_data_notification(
                                    MailData {
                                        inbox_metadata: InboxMetadata {
                                            inbox_count: 130,
                                            inbox_unread: 1,
                                            others_count: 0,
                                            others_unread_count: 0,
                                        },
                                        quota: Default::default(),
                                        messages: vec![]
                                      //  messages: vec![MailDataMessage::new(Local::now().to_utc(), EmailAddress::from_str("tachyon@tachyon.internal").unwrap(), "System".into(), "msgid1".into(),"Verify your account".into(), 123, false)],
                                    }
                                )),
                            });

                            notif_sender.send(initial_mail_data).await?;


                            /*let initial_mail_data = NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: DisplayName::new_from_ref("Hotmail"),
                                payload: MsgPayload::Raw(RawMsgPayloadFactory::get_mail_data_notification(EmailAddress::from_str("tachyon@tachyon.internal").unwrap(), "System".to_string(), "Verify your account".into())),
                            });

                            notif_sender.send(initial_mail_data).await?;*/


                            let notttt = NotificationServerCommand::NOT(NotServer {
                                payload: NotificationPayloadType::Normal(NotificationFactory::alert(&msn_user.uuid, msn_user.get_email_address(), "Device verification required ! Verification is necessary to access encrypted messages and to stay secure.", "http://127.0.0.1:8080/tachyon", "/verify?test=1", "/verify?test=2", Some("tachyon_logo_2.png"))),
                            });



                            //notif_sender.send(notttt).await;


                            let recipient_pid = format!("0x{}:0x{}", &msn_user.uuid.get_least_significant_bytes_as_hex(), &msn_user.uuid.get_most_significant_bytes_as_hex());

                            //Icon tag needs to be present or no other images are loaded :))))))) AAAAAAAh
                            //Icon image must be 48x48 or less, or it will be replace by a placeholder (a bell icon)
                            //Icon image name: if it contains _32x32.png it gets replaced to _48x32.png xD
                            //Icon url depends of the domain from the siteurl. It ignores all the path element from the siteUrl and it happens a / before
                            // ie: siteurl = http://127.0.0.1:8080/ads. you still needs to pust icon="ads/youricon.png"
                            // ==> it will call http://127.0.0.1:8080/ads/youricon.png
                            // Color seems to be their weird inverted RGBA again, like text messages.

                            //TODO handle alpha in FontColor
                            let color = format!("{alpha}{bgr}", alpha = "CF", bgr = FontColor::parse_from_rgb("FF03FF").unwrap().serialize_bgr());

                            let testo2 = format!(r##"<NOTIFICATION id="1" siteid="45705" siteurl="http://contacts.msn.com"  >
  <TO pid="{pid}" name="{email}">
    <VIA agent="messenger" />
  </TO>
  <MSG id="1" siteurl="http://127.0.0.1:8080/ads">
    <SUBSCR url="http://contacts.msn.com/s.htm" />
    <ACTION url="http://contacts.msn.com/a.htm" />
    <WINDOW minimizeByDefault="false" />
    <BODY icon="blahblahblah">
      <TEXT>aeon.shl@shl.local</TEXT>
<TEXTXML>
  <TP ID="104" >
     <E1 B="{color}"></E1>
     <I1 I="http://127.0.0.1:8080/ads/alert-background.png" ></I1>
     <L1 L="http://127.0.0.1:8080/tachyon" F="FFFFFFFF"></L1>
     <I2 I="http://127.0.0.1:8080/ads/spongebob-icon.png" ></I2>
     <I3 I="http://127.0.0.1:8080/ads/spongebob-icon.png" ></I3>
     <T1 T="DEVELOPPERS DEVELOPPERS DEVELOPPERS" F="{color}"></T1>
  </TP>
</TEXTXML>
    </BODY>
  </MSG>
</NOTIFICATION>"##, pid = recipient_pid, email = &msn_user.get_email_address(), color = color);

                            let wtf2 = NotificationServerCommand::NOT(NotServer {
                                payload: NotificationPayloadType::Raw(testo2),
                            });

                            notif_sender.send(wtf2).await;

                        }
                    }
                },
                _ => {
                    //return unauth error
                    todo!()
                }

            }
            Ok(())
        },

        _ => {todo!()}
    }

}

async fn handle_ready(raw_command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, client_data: TachyonClient, local_store: &mut LocalClientData) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => handle_usr(command, local_store.email_addr.clone(), command_sender).await,
        NotificationClientCommand::PNG => handle_png(command_sender).await,
        NotificationClientCommand::ADL(command) => handle_adl(command, client_data, command_sender).await,
        NotificationClientCommand::RML(command) => handle_rml(command, client_data, command_sender).await,
        NotificationClientCommand::UUX(command) => handle_uux(command, local_store, command_sender).await,
        NotificationClientCommand::UUM(command) => handle_uum(command, client_data, command_sender).await,
        NotificationClientCommand::XFR(command) => handle_xfr(command, local_store, command_sender).await,
        NotificationClientCommand::BLP(command) => {
            command_sender.send(NotificationServerCommand::BLP(command)).await?;
            Ok(())
        }
        NotificationClientCommand::CHG(command) => handle_chg(command, local_store, client_data, command_sender).await,
        NotificationClientCommand::PRP(command) => handle_prp(command, local_store, client_data, command_sender).await,
        NotificationClientCommand::UUN(_command) => {Ok(())},
        NotificationClientCommand::RAW(command) => {
            warn!("Received RAW command: {:?}", command);
            Ok(())
        },
        NotificationClientCommand::PUT(command) => handle_put(command, local_store, client_data, command_sender).await,
        NotificationClientCommand::OUT => {Ok(())}
        NotificationClientCommand::VER(_) => {Ok(())}
        NotificationClientCommand::CVR(_) => {Ok(())}
        NotificationClientCommand::FQY(command) => {handle_fqy(command, client_data, command_sender).await}
        NotificationClientCommand::SDG(_) => {Ok(())}
        NotificationClientCommand::URL(command) => {
            handle_url(command, local_store, client_data, command_sender).await
        }
    }
    }

