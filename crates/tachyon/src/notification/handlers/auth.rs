use std::time::Duration;
use crate::matrix;
use crate::matrix::cross_signing::{ check_device_is_crossed_signed, check_secret_storage_state};
use crate::matrix::sync::sync;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::alert::{Alert, AlertError, AlertSuccess};
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global::global_state::GlobalState;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::tachyon::repository::RepositoryStr;
use anyhow::Error;
use log::debug;
use matrix_sdk::Client;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::{NotServer, NotificationPayloadType};
use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrServer};
use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use tokio::sync::mpsc::Sender;
use tokio::{select, task};
use crate::matrix::cross_signing;
use crate::matrix::services::login::MatrixLoginService;
use crate::notification::models::notification_handle::NotificationHandle;
use crate::tachyon::global::tachyon_config::TachyonConfig;

const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"1000\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"100\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";

pub(crate) async fn handle_auth(command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, tachyon_state: &GlobalState, local_store: &mut LocalClientData, config: &TachyonConfig, matrix_login_service: &Box<dyn MatrixLoginService>) -> Result<(), anyhow::Error> {
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

                        SsoPhaseClient::S { ticket_token: ticket_token, challenge: _, endpoint_guid } => {

                            let user_id = local_store.email_addr.to_owned_user_id();

                            let matrix_token = tachyon_state.secret_encryptor().decrypt(ticket_token.as_str())?;

                            let matrix_client = matrix_login_service.login_with_token(&user_id, &matrix_token, !config.strict_ssl).await?;

                            let endpoint_id = EndpointId::new(local_store.email_addr.clone(), Some(endpoint_guid));
                            let msn_user = MsnUser::new(endpoint_id);


                            let tachyon_client = TachyonClient::new(matrix_client.clone(), config.clone(), msn_user.clone(), ticket_token.clone(), NotificationHandle::new(notif_sender.clone()), local_store.client_shutdown_snd.clone(), local_store.client_shutdown_recv.resubscribe());
                            let drop_guard = tachyon_state.insert_clients(ticket_token.as_str().to_owned(), tachyon_client.clone());

                            local_store.client_drop_guard = Some(drop_guard);
                            local_store.token = ticket_token.clone();
                            local_store.tachyon_client = Some(tachyon_client.clone());
                            local_store.matrix_client = Some(matrix_client.clone());
                            local_store.phase = ConnectionPhase::Ready;

                            sync_with_server_task(&notif_sender, local_store, &ticket_token, &matrix_client, &msn_user, tachyon_client, config)?;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: local_store.email_addr.clone(),
                                verified: true,
                                unknown_arg: false,
                            });
                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;


                            /*let initial_mail_data = NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: DisplayName::new_from_ref("Hotmail"),
                                payload: MsgPayload::Raw(RawMsgPayloadFactory::get_mail_data_notification(EmailAddress::from_str("tachyon@tachyon.internal").unwrap(), "System".to_string(), "Verify your account".into())),
                            });

                            notif_sender.send(initial_mail_data).await?;*/

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

fn sync_with_server_task(notif_sender: &Sender<NotificationServerCommand>, local_store: &LocalClientData, ticket_token: &TicketToken, matrix_client: &Client, msn_user: &MsnUser, tachyon_client: TachyonClient, config: &TachyonConfig) -> Result<(), Error> {
    let msn_user_clone = msn_user.clone();
    let matrix_client_clone = matrix_client.clone();
    let notif_sender_clone = notif_sender.clone();
    let ticket_token_clone = ticket_token.clone();
    let config_clone = config.clone();
    let client_shutdown_snd = local_store.client_shutdown_snd.clone();
    let mut client_shutdown_recv = local_store.client_shutdown_recv.resubscribe();


    task::spawn(async move {
        let cross_signed = check_device_is_crossed_signed(&matrix_client_clone).await.unwrap();

        if !cross_signed {

            let sign_sync_loop_kill_snd = cross_signing::cross_sign_sync_task(&matrix_client_clone, client_shutdown_recv.resubscribe()).await.unwrap();

            let notification_id = rand::random::<i32>();

            let verif_not = NotificationServerCommand::NOT(NotServer {
                payload: NotificationPayloadType::Normal(NotificationFactory::alert(&msn_user_clone.uuid, msn_user_clone.get_email_address(), "Oops ! Your device is not verified yet ! Click here to verify.", format!("http://127.0.0.1:{}/tachyon", config_clone.http_port).as_str(), format!("http://127.0.0.1:{}/tachyon/confirm_device?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), format!("http://127.0.0.1:{}/tachyon/confirm_device?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), Some("shield_verify.png"), notification_id)),
            });

            let (alert, receiver) = Alert::new_confirm_device(Duration::from_mins(5));
            tachyon_client.alerts().insert(notification_id, alert);

            notif_sender_clone.send(verif_not).await;

            select! {
                recv = receiver.recv() => {
                   let _ = sign_sync_loop_kill_snd.send(()).await;
                    match recv {
                        Ok(success) => {
                            //We recheck if the device is cross signed as we can have false positives and the user needs to reset his cryptographic identity in such cases.
                            if check_device_is_crossed_signed(&matrix_client_clone).await.unwrap() {

                            } else {
                                let _  = client_shutdown_snd.send(());
                                return;
                            }
                        }
                        Err(err) => {
                            let _  = client_shutdown_snd.send(());
                            debug!("error received stopping sync_with_server_task");
                            return;
                        }
                    }
                },
                kill_recv = client_shutdown_recv.recv() => {
                    debug!("client_kill_recv stopping sync_with_server_task");
                    let _ = sign_sync_loop_kill_snd.send(()).await;
                    return;
                }
            }
        }

/*        let secret_storage_enabled = check_secret_storage_state(&matrix_client_clone).await.unwrap();

        if !secret_storage_enabled {

            let notification_id = rand::random::<i32>();


            let backup_not = NotificationServerCommand::NOT(NotServer {
                payload: NotificationPayloadType::Normal(NotificationFactory::alert(&msn_user_clone.uuid, msn_user_clone.get_email_address(), "Account backup is disabled. Click here to set it up !", format!("http://127.0.0.1:{}/tachyon", config_clone.http_port).as_str(), format!("http://127.0.0.1:{}/tachyon/backup?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), format!("http://127.0.0.1:{}/tachyon/backup?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), None, notification_id)),
            });

            notif_sender_clone.send(backup_not).await;

        }*/


        notif_sender_clone.send(NotificationServerCommand::RAW(RawCommand::without_payload("SBS 0 null"))).await;

        //This makes the client login to succeed.
        let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
            sender: "Hotmail".to_string(),
            display_name: DisplayName::new_from_ref("Hotmail"),
            payload: MsgPayload::Raw(RawMsgPayloadFactory::get_msmsgs_profile(
                &msn_user_clone.uuid.get_puid(),
                msn_user_clone.get_email_address(),
                &ticket_token_clone,
            )),
        });

        notif_sender_clone.send(initial_profile_msg).await;

        //Todo fetch endpoint data
        let endpoint_data = b"<Data></Data>";
        notif_sender_clone
            .send(NotificationServerCommand::RAW(RawCommand::with_payload(
                &format!("UBX 1:{}", &msn_user_clone.get_email_address().as_str()),
                endpoint_data.to_vec(),
            )))
            .await;

        //Todo check the device state before we sync

        let sync_join_handle = sync(tachyon_client, matrix_client_clone, client_shutdown_snd, client_shutdown_recv).await;

        let initial_mail_data = NotificationServerCommand::MSG(MsgServer {
            sender: "Hotmail".to_string(),
            display_name: DisplayName::new_from_ref("Hotmail"),
            payload: MsgPayload::Raw(RawMsgPayloadFactory::get_initial_mail_data_empty_notification()),
        });

        notif_sender_clone.send(initial_mail_data).await;
    });
    Ok(())
}