use std::time::Duration;
use crate::matrix::cross_signing::{self, check_device_is_crossed_signed};
use crate::matrix::services::login::{domain_user_id_to_owned, persist_session, store_hydration_warning};
use crate::matrix::sync::sync;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::session_state::SessionState;
use crate::notification::notification_server::ConnectionShell;
use crate::tachyon::alert::{Alert};
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::config::paths::get_store_path;
use crate::tachyon::global_state::{GlobalState};
use log::{debug, info, warn};
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
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use tachyon_sqlite_store::HomeserverUrl;
use tokio::sync::mpsc::Sender;
use tokio::{select, task};
use crate::tachyon::config::tachyon_config::TachyonConfig;

const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"1000\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"100\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";

pub(crate) async fn handle_auth(command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, tachyon_state: &GlobalState, session_state: &mut SessionState, shell: &mut ConnectionShell, config: &TachyonConfig) -> Result<(), anyhow::Error> {
    match command {
        NotificationClientCommand::USR(command) => {
            match command.auth_type {
                AuthOperationTypeClient::Sso(content) => {
                    match content {
                        SsoPhaseClient::I { email_addr } => {
                            session_state.email_addr = email_addr;
                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU".to_string() }));
                            let gcf_response = RawCommand::with_payload("GCF 0", SHIELDS_PAYLOAD.as_bytes().to_vec());

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.send(NotificationServerCommand::RAW(gcf_response)).await?;
                        },

                        SsoPhaseClient::S { ticket_token: ticket_token, challenge: _, endpoint_guid } => {
                            let token = ticket_token.as_str().to_string();
                            session_state.token = ticket_token.clone();

                            let msn_user = MsnUser::new(EndpointId::new(session_state.email_addr.clone(), Some(endpoint_guid.clone())));

                            // registered before the OK: the client fires its first ABCH SOAP right after, and the lookup must resolve.
                            // The backend stays pending until the matrix login lands.
                            let client = TachyonClient::new(
                                config.clone(),
                                msn_user.clone(),
                                ticket_token.clone(),
                                notif_sender.clone(),
                                shell.client_shutdown_snd.clone(),
                                shell.client_shutdown_recv.resubscribe(),
                                Default::default(),
                            );
                            shell.client_drop_guard = Some(tachyon_state.insert_clients(token.clone(), client.clone()));
                            session_state.tachyon_client = Some(client.clone());
                            session_state.phase = ConnectionPhase::Ready;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: session_state.email_addr.clone(),
                                verified: true,
                                unknown_arg: false,
                            });
                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;

                            // off the read loop: its SDK round trips can park for seconds while PNG/ADL must keep flowing
                            spawn_restore_or_toast(tachyon_state.clone(), token.clone(), client, config.clone());
                        }
                    }
                },
                _ => {
                    //TODO unauthenticated error reply
                    warn!("MSNP|NS: unhandled USR auth operation during authentication");
                }
            }
            Ok(())
        },

        _ => {
            //TODO unauthenticated error reply
            warn!("MSNP|NS: unhandled command during authentication");
            Ok(())
        }
    }
}

fn spawn_restore_or_toast(state: GlobalState, token: String, client: TachyonClient, config: TachyonConfig) {
    task::spawn(async move {
        let credentials = match state.credentials().load(&token) {
            Ok(Some(credentials)) => credentials,
            Ok(None) => {
                send_login_toast(&client).await;
                return;
            }
            Err(error) => {
                warn!("credentials unreadable for {}, asking for sign-in: {}", token, error);
                send_login_toast(&client).await;
                return;
            }
        };

        if let Ok(user_id) = domain_user_id_to_owned(credentials.user_id()) {
            // probe before build: the builder creates the sqlite files eagerly and would mask this
            if let Some(warning) = store_hydration_warning(&get_store_path(&user_id)) {
                warn!("{}", warning);
            }
        }

        let config_homeserver = config.homeserver_url.as_deref().and_then(|url| HomeserverUrl::parse(url).ok());
        match state.matrix_login_service().restore_saved_session(&credentials, config_homeserver.as_ref(), !config.strict_ssl).await {
            Ok(matrix_client) => {
                info!("restored the saved session for {}", token);
                persist_session(&matrix_client, &token, state.credentials());
                client.attach_backend(matrix_client, state.credentials_handle());
            }
            outcome => {
                info!("saved session for {} not restored ({:?}); asking for sign-in", token, outcome);
                send_login_toast(&client).await;
            }
        }
    });
}

// The one startup alert, pointing at the section that is missing
pub(crate) async fn send_login_toast(client: &TachyonClient) {
    let msn_user = client.own_user();
    let email = msn_user.get_email_address();
    let (username, homeserver) = email.as_str().split_once('@').map(|(username, homeserver)| (username.to_string(), homeserver.to_string())).unwrap_or_default();
    let login_url = format!(
        "http://127.0.0.1:{}/tachyon/login?username={}&homeserver={}&t={}",
        client.config().http_port, username, homeserver, client.ticket_token().as_str()
    );
    let notification_id: i32 = rand::random();

    let secret_not = NotificationServerCommand::NOT(NotServer {
        payload: NotificationPayloadType::Normal(NotificationFactory::alert(
            &MsnUser::new(EndpointId::new(email.clone(), None)).uuid,
            &email,
            "Sign in required - click here to continue",
            login_url.as_str(),
            "",
            "",
            None,
            notification_id,
        )),
    });
    // Err = receiver dropped, client gone: the toast is undeliverable and
    // nothing follows in this flow (§8 D2).
    if let Err(error) = client.notification_handle().send(secret_not).await {
        warn!("MSNP|NS: login toast dropped, client gone: {}", error);
    }
}

pub(crate) fn sync_with_server_task(tachyon_client: TachyonClient, matrix_client: &Client) {
    let msn_user_clone = tachyon_client.own_user();
    let matrix_client_clone = matrix_client.clone();
    let notif_sender_clone = tachyon_client.notification_handle();
    let ticket_token_clone = tachyon_client.ticket_token();
    let config_clone = tachyon_client.config();
    let client_shutdown_snd = tachyon_client.shutdown_sender();
    let mut client_shutdown_recv = tachyon_client.shutdown_receiver();


    task::spawn(async move {
        let cross_signed = check_device_is_crossed_signed(&matrix_client_clone).await.unwrap();

        if !cross_signed {

            let sign_sync_loop_kill_snd = cross_signing::cross_sign_sync_task(&matrix_client_clone, client_shutdown_recv.resubscribe()).await.unwrap();

            let notification_id = rand::random::<i32>();

            let verif_not = NotificationServerCommand::NOT(NotServer {
                payload: NotificationPayloadType::Normal(NotificationFactory::alert(&msn_user_clone.uuid, msn_user_clone.get_email_address(), "Device verification required - click here to continue", format!("http://127.0.0.1:{}/tachyon", config_clone.http_port).as_str(), format!("http://127.0.0.1:{}/tachyon/confirm_device?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), format!("http://127.0.0.1:{}/tachyon/confirm_device?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), Some("shield_verify.png"), notification_id)),
            });

            let (alert, receiver) = Alert::new_confirm_device(Duration::from_mins(5));
            tachyon_client.alerts().insert(notification_id, alert);

            // Err = receiver dropped, client gone: stop producing the
            // login-sequence commands (§8 D2).
            if let Err(error) = notif_sender_clone.send(verif_not).await {
                warn!("MSNP|NS: verification alert dropped, client gone: {}", error);
                return;
            }

            // the login page can set the identity up without ever touching the alert; a recheck tick un-parks this wait
            let (recheck_snd, mut recheck_rcv) = tokio::sync::mpsc::channel::<()>(1);
            let recheck_client = matrix_client_clone.clone();
            task::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    if check_device_is_crossed_signed(&recheck_client).await.unwrap_or(false) {
                        let _ = recheck_snd.send(()).await;
                        break;
                    }
                }
            });

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
                },
                _ = recheck_rcv.recv() => {
                    let _ = sign_sync_loop_kill_snd.send(()).await;
                }
            }
        }

        if let Err(error) = notif_sender_clone.send(NotificationServerCommand::RAW(RawCommand::without_payload("SBS 0 null"))).await {
            warn!("MSNP|NS: SBS dropped, client gone: {}", error);
            return;
        }

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

        if let Err(error) = notif_sender_clone.send(initial_profile_msg).await {
            warn!("MSNP|NS: initial profile MSG dropped, client gone: {}", error);
            return;
        }

        //Todo fetch endpoint data
        let endpoint_data = b"<Data></Data>";
        if let Err(error) = notif_sender_clone
            .send(NotificationServerCommand::RAW(RawCommand::with_payload(
                &format!("UBX 1:{}", &msn_user_clone.get_email_address().as_str()),
                endpoint_data.to_vec(),
            )))
            .await
        {
            warn!("MSNP|NS: initial UBX dropped, client gone: {}", error);
            return;
        }

        //Todo check the device state before we sync

        let sync_join_handle = sync(tachyon_client.clone(), matrix_client_clone.clone(), client_shutdown_snd, client_shutdown_recv).await;

        let initial_mail_data = NotificationServerCommand::MSG(MsgServer {
            sender: "Hotmail".to_string(),
            display_name: DisplayName::new_from_ref("Hotmail"),
            payload: MsgPayload::Raw(RawMsgPayloadFactory::get_initial_mail_data_empty_notification()),
        });

        // Tail of the task: nothing left to produce, so the client-gone case
        // only needs the log (§8 D2).
        if let Err(error) = notif_sender_clone.send(initial_mail_data).await {
            warn!("MSNP|NS: initial mail data MSG dropped, client gone: {}", error);
        }
    });
}
