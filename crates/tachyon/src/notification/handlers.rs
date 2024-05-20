use std::path::Path;
use std::str::FromStr;

use anyhow::{anyhow, Error};
use lazy_static_include::syn::BinOp::Ne;
use log::{debug, error, warn};
use matrix_sdk::RoomMemberships;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;

use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::nfy::{NfyOperation, NfyServer};
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::msnp::notification::command::usr::{AuthPolicy, OperationTypeClient, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrServer};
use msnp::msnp::notification::command::uum::UumPayload;
use msnp::msnp::notification::command::uux::UuxPayload;
use msnp::msnp::notification::models::endpoint_data::EndpointData;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::models::capabilities::ClientCapabilities;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::network_id::NetworkId;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use msnp::shared::models::uuid::Uuid;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use msnp::shared::payload::msg::text_msg::FontStyle;
use msnp::shared::payload::nfy::nfy_put_payload::RawNfyPayload;

use crate::{matrix, notification};
use crate::matrix::msn_user_resolver;
use crate::matrix::sync::initial_sync;
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::notification::notification_server::{LocalStore, Phase};
use crate::shared::identifiers::{MatrixDeviceId, MatrixIdCompatible};

pub(crate) async fn handle_negotiation(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, mut local_store: &mut LocalStore) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::VER(command) => {
            if command.first_candidate != MSNP18 && command.second_candidate != MSNP18 {
                //Unsupported protocol version
                //TODO add error code
                notif_sender.send(NotificationServerCommand::OUT).await?;
                return Ok(());
            }

            notif_sender.send(NotificationServerCommand::VER(command.get_response_for(MSNP18))).await;
            Ok(())
        },
        NotificationClientCommand::CVR(command) => {
            local_store.phase = crate::notification::notification_server::Phase::Authenticating;
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
            Ok(())
        },
        _ => {
            Err(anyhow!("WTF are you doing here"))
        }
    }
}


const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";

pub(crate) async fn handle_auth(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, client_store: &ClientStoreFacade, mut local_store: &mut LocalStore, kill_signal: &broadcast::Receiver<()>) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => {
            match command.auth_type {
                OperationTypeClient::Sso(content) => {
                    match content {
                        SsoPhaseClient::I { email_addr } => {
                            local_store.email_addr = email_addr;
                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU".to_string() }));
                            let gcf_response = RawCommand::with_payload("GCF 0", SHIELDS_PAYLOAD.as_bytes().to_vec());

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.send(NotificationServerCommand::RAW(gcf_response)).await?;
                        },

                        SsoPhaseClient::S { ticket_token, challenge, endpoint_guid } => {

                            let user_id = local_store.email_addr.to_owned_user_id();

                            let device_id = format!("tachyon3-{}", MatrixDeviceId::from_hostname()?.to_string());


                            let matrix_client = matrix::login::login(user_id, device_id, ticket_token.clone(), &Path::new(format!("C:\\temp\\{}", &local_store.email_addr).as_str()), None, true).await?;

                            let endpoint_id = EndpointId::new(local_store.email_addr.clone(), Some(endpoint_guid));
                            let msn_user = MsnUser::new(endpoint_id);

                            let client_data = ClientData::new(msn_user.clone(), ticket_token.clone(), matrix_client.clone());
                            client_store.insert_client_data(ticket_token.as_str().to_owned(), client_data.clone());

                            local_store.token = ticket_token.clone();
                            local_store.client_data = Some(client_data.clone());
                            local_store.phase = Phase::Ready;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: local_store.email_addr.clone(),
                                verified: true,
                                unknown_arg: false,
                            });

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.clone().send(NotificationServerCommand::RAW(RawCommand::without_payload("SBS 0 null"))).await?;

                            let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: "Hotmail".to_string(),
                                payload: MsgPayload::Raw(RawMsgPayloadFactory::get_msmsgs_profile(&msn_user.uuid.get_puid(), &local_store.email_addr, &ticket_token))
                            });

                            notif_sender.send(initial_profile_msg).await?;

                            let notif_sender_ed = notif_sender.clone();
                            let email_addr = msn_user.get_email_address().clone();
                            tokio::spawn(async move {
                                //Todo fetch endpoint data
                                let endpoint_data = b"<Data></Data>";
                                notif_sender_ed.send(NotificationServerCommand::RAW(RawCommand::with_payload(&format!("UBX 1:{}", &email_addr.as_str()), endpoint_data.to_vec()))).await;
                            });
                        }

                    }
                },
                OperationTypeClient::Sha(_) => {
                    //return unauth error
                    todo!()
                }

            }
            Ok(())
        },

        _ => {todo!()}
    }

}


pub(crate) async fn handle_command(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, client_data: ClientData, mut local_store: &mut LocalStore, kill_signal: &broadcast::Receiver<()>) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => {
            match command.auth_type {

                OperationTypeClient::Sso(_) => {
                    todo!()
                    //return error;
                },
                OperationTypeClient::Sha(phase) => {

                    let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                        email_addr: local_store.email_addr.clone(),
                        verified: true,
                        unknown_arg: false,
                    });
                    notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                }

            }
            Ok(())
        },
        NotificationClientCommand::PNG => {
            notif_sender.send(NotificationServerCommand::QNG(60)).await?;
            Ok(())
        }
        //The client waits indefinitely for initial ADL Response, useful if we need time to sync contacts without hitting the timeout :D
        NotificationClientCommand::ADL(command) => {

            debug!("ADL: {:?}", &command);

            let contacts = command.payload.get_contacts()?;

            client_data.inner.contact_list.lock().unwrap().add_contacts(contacts, command.payload.is_initial());

            notif_sender.send(NotificationServerCommand::Ok(command.get_ok_response("ADL"))).await?;

            Ok(())
        }
        NotificationClientCommand::RML(command) => {
            debug!("RML: {:?}", &command);

            client_data.inner.contact_list.lock().unwrap().remove_contacts(command.payload.get_contacts()?);
            notif_sender.send(NotificationServerCommand::Ok(command.get_ok_response("RML"))).await?;

            Ok(())
        }
        NotificationClientCommand::UUX(command) => {
            let ok_resp = command.get_ok_response();

            match command.payload {
                None => {}
                Some(payload) => {
                    match payload {
                        UuxPayload::PrivateEndpointData(private_endpoint_data) => {
                            local_store.private_endpoint_data = private_endpoint_data;
                            //TODO
                        }
                        UuxPayload::Unknown(_) => {}
                    }
                }
            }

            notif_sender.send(NotificationServerCommand::Uux(ok_resp)).await?;
            Ok(())
        },
        NotificationClientCommand::UUM(command) => {

            let ok_response = command.get_ok_response();

            match command.payload {
                UumPayload::TextMessage(content) => {
                    let matrix_client = client_data.get_matrix_client();
                    let room = matrix_client.get_dm_room(&command.destination.email_addr.to_owned_user_id());
                    match room {
                        None => {
                            //NO DM ROOM FOUND
                        }
                        Some(room) => {
                            //TODO SMILEY TO EMOJI
                            //TODO Store event id for dedup

                            let content = if content.is_styling_default() {
                                RoomMessageEventContent::text_plain(content.body)
                            } else {
                                let mut message = content.body.clone();

                                if !content.is_default_font_styles() {
                                    if content.font_styles.matches(FontStyle::Bold) {
                                        message = format!("<b>{}</b>", message)
                                    }

                                    if content.font_styles.matches(FontStyle::Italic) {
                                        message = format!("<i>{}</i>", message)
                                    }

                                    if content.font_styles.matches(FontStyle::Underline) {
                                        message = format!("<u>{}</u>", message)
                                    }

                                    if content.font_styles.matches(FontStyle::StrikeThrough) {
                                        message = format!("<strike>{}</strike>", message)
                                    }
                                }

                                let color_attr = if content.is_default_font_color() { String::new() } else { format!(" color=\"{}\"", content.font_color.serialize_rgb())};
                                let face_attr = if content.is_default_font() { String::new() } else { format!(" face=\"{}\"", content.font_family) };
                                message = format!("<font{}{}>{}</font>",  color_attr, face_attr, message);

                                RoomMessageEventContent::text_html(content.body, message)
                            };

                            let response = room.send(content).await?;
                            //self.add_to_events_sent(response.event_id.to_string());
                            notif_sender.send(NotificationServerCommand::Ok(ok_response)).await?;
                        }
                    }

                    Ok(())
                },
                UumPayload::TypingUser(_) => {
                    todo!()
                }
                UumPayload::Nudge(_) => {
                    todo!()

                }
                UumPayload::Raw(_) => {
                    todo!()
                }
            }
        }
        NotificationClientCommand::BLP(command) => {
            notif_sender.send(NotificationServerCommand::BLP(command)).await?;
            Ok(())
        }
        NotificationClientCommand::CHG(command) => {
            if local_store.needs_initial_presence {
                local_store.needs_initial_presence = false;

                let notif_sender = notif_sender.clone();
                let client_data = client_data.clone();
                let kill_signal = kill_signal.clone();

                tokio::spawn(async move {
                    let initial_sync_result = initial_sync(command.tr_id, &client_data).await;
                    if let Err(err) = initial_sync_result.as_ref() {
                        error!("An error occured during initial sync: {}", err);
                        //TODO return a real error instead of outing the client
                        let _result = notif_sender.send(NotificationServerCommand::OUT).await;
                    }

                    let (mut iln, mut notifications) = initial_sync_result.expect("to be here");

                    for current in iln.drain(..) {
                        let _result = notif_sender.send(NotificationServerCommand::ILN(current)).await;
                    }

                    for current in notifications.drain(..) {
                        let _result = notif_sender.send(NotificationServerCommand::NOT(current)).await;
                    }
                });
            }

            notif_sender.send(NotificationServerCommand::CHG(command.clone())).await?;

            Ok(())
        }
        NotificationClientCommand::PRP(command) => {Ok(())}
        NotificationClientCommand::UUN(command) => {Ok(())}
        NotificationClientCommand::XFR() => {Ok(())}
        NotificationClientCommand::RAW(command) => {
            warn!("Received RAW command: {:?}", command);
            Ok(())
        },
        NotificationClientCommand::PUT(command) => {



            let ok = command.get_ok_command();
            notif_sender.send(NotificationServerCommand::PUT(ok)).await?;

            let mut payload = command.payload;
            payload.envelope.swap_sides();
            payload.envelope.flags = None;

            notif_sender.send(NotificationServerCommand::NFY(NfyServer {
                operation: NfyOperation::Put,
                payload
            })).await?;

            Ok(())
        },
        NotificationClientCommand::OUT => {Ok(())}
        _ => {
            warn!("Received unknown command");
            Ok(())
        }
    }
}

