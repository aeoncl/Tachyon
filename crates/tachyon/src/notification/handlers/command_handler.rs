use crate::matrix;
use crate::matrix::direct_service::DirectService;
use crate::matrix::sync2::{build_sliding_sync, sync};
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::notification::handlers::adl_handler::handle_adl;
use crate::notification::handlers::chg_handler::handle_chg;
use crate::notification::handlers::png_handler::handle_png;
use crate::notification::handlers::put_handler::handle_put;
use crate::notification::handlers::rml_handler::handle_rml;
use crate::notification::handlers::usr_handler::handle_usr;
use crate::notification::handlers::uum_handler::handle_uum;
use crate::notification::handlers::uux_handler::handle_uux;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;
use crate::shared::identifiers::MatrixIdCompatible;
use anyhow::anyhow;
use log::{debug, error, warn};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::sleep::sleep;
use matrix_sdk_ui::sync_service::SyncService;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::nfy::{NfyOperation, NfyServer};
use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrServer};
use msnp::msnp::notification::command::uum::UumPayload;
use msnp::msnp::notification::command::uux::UuxPayload;
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use msnp::shared::payload::msg::text_msg::FontStyle;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::task;

pub(crate) async fn handle_command(command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, client_store: &ClientStoreFacade, local_client_data: &mut LocalClientData) -> Result<(), anyhow::Error> {

    let command_result = match &local_client_data.phase {
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


const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";
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

                        SsoPhaseClient::S { ticket_token, challenge, endpoint_guid } => {

                            let user_id = local_store.email_addr.to_owned_user_id();

                            let matrix_client = matrix::login::login_with_token(user_id, ticket_token.clone(), true).await?;
                            let sync_service = SyncService::builder(matrix_client.clone()).build().await?;
                            let sliding_sync = build_sliding_sync(&matrix_client).await?;
                            let direct_service = DirectService::new(matrix_client.clone());
                            direct_service.init().await.unwrap();


                            let endpoint_id = EndpointId::new(local_store.email_addr.clone(), Some(endpoint_guid));
                            let msn_user = MsnUser::new(endpoint_id);


                            let client_data = ClientData::new(msn_user.clone(), ticket_token.clone(), notif_sender.clone(), matrix_client.clone(), sliding_sync, sync_service, direct_service);
                            client_store.insert_client_data(ticket_token.as_str().to_owned(), client_data.clone());

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

                            sync(client_data, local_store.client_kill_recv.resubscribe());
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

async fn handle_ready(raw_command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, client_data: ClientData, local_store: &mut LocalClientData) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => handle_usr(command, local_store.email_addr.clone(), command_sender).await,
        NotificationClientCommand::PNG => handle_png(command_sender).await,
        NotificationClientCommand::ADL(command) => handle_adl(command, client_data, command_sender).await,
        NotificationClientCommand::RML(command) => handle_rml(command, client_data, command_sender).await,
        NotificationClientCommand::UUX(command) => handle_uux(command, local_store, command_sender).await,
        NotificationClientCommand::UUM(command) => handle_uum(command, client_data, command_sender).await,
        NotificationClientCommand::BLP(command) => {
            command_sender.send(NotificationServerCommand::BLP(command)).await?;
            Ok(())
        }
        NotificationClientCommand::CHG(command) => handle_chg(command, local_store, client_data, command_sender).await,
        NotificationClientCommand::PRP(command) => {Ok(())}
        NotificationClientCommand::UUN(command) => {Ok(())}
        NotificationClientCommand::XFR() => {Ok(())}
        NotificationClientCommand::RAW(command) => {
            warn!("Received RAW command: {:?}", command);
            Ok(())
        },
        NotificationClientCommand::PUT(command) => handle_put(command, local_store, client_data, command_sender).await,
        NotificationClientCommand::OUT => {Ok(())}
        _ => {
            warn!("Received unknown command");
            Ok(())
        }
    }
    }