use crate::matrix::cross_signing;
use crate::matrix::cross_signing::check_device_is_crossed_signed;
use crate::matrix::sync::sync;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::alert::{Alert, AlertSuccess};
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::global_state::{GlobalState, PendingLogin};
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use anyhow::{anyhow, Error};
use log::{debug, error, warn};
use matrix_sdk::Client;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::{NotServer, NotificationPayloadType};
use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrServer};
use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use std::sync::Arc;
use std::time::Duration;
use tachyon_backend_matrix::infrastructure::backend::BackendSessionMatrix;
use tachyon_core::application::error::AuthError;
use tachyon_core::domain::auth::{BridgeMetadata, InteractiveAuthStarted};
use tachyon_core::domain::ids::UserId as CoreUserId;
use tokio::sync::mpsc::Sender;
use tokio::time::{sleep_until, timeout_at, Instant};
use tokio::{select, task};

const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"1000\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"100\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";

/// Once the client has its `USR OK` it sits on the "signing in" screen waiting for the intial profile `MSG`.
/// rendering any `NOT` alert we send, for about five minutes before it gives up. That wait
/// is the only window we get to ask the user for something out-of-band.
///
/// Every such step — an interactive login, then device verification — has to fit inside
/// that one budget, so the deadline is computed once per sign-in and shared, rather than
/// each step claiming five minutes of its own.
const CLIENT_SIGN_IN_WINDOW: Duration = Duration::from_secs(5 * 60);

pub(crate) async fn handle_auth(command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, tachyon_state: &GlobalState, local_store: &mut LocalClientData, config: &TachyonConfig) -> Result<(), anyhow::Error> {
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

                            let sign_in_deadline = Instant::now() + CLIENT_SIGN_IN_WINDOW;
                            let email_addr = local_store.email_addr.clone();

                            // The ticket is derived from the address, so a mismatch means it
                            // was not issued by this instance for this account.
                            let expected_ticket = tachyon_state.ticket_for(&email_addr);
                            if ticket_token.as_str() != expected_ticket.as_str() {
                                return Err(anyhow!("Ticket token does not match {}", email_addr.as_str()));
                            }

                            let endpoint_id = EndpointId::new(email_addr.clone(), Some(endpoint_guid));
                            let msn_user = MsnUser::new(endpoint_id);

                            // Accept the sign-in before doing any of the slow work: the client
                            // then parks on its "signing in" screen until SBS arrives, and that
                            // wait is the window we get for anything the user must do
                            // out-of-band.
                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: email_addr.clone(),
                                verified: true,
                                unknown_arg: false,
                            });
                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;

                            let matrix_client = authenticate(
                                tachyon_state,
                                &notif_sender,
                                &email_addr,
                                &msn_user,
                                config,
                                sign_in_deadline,
                            ).await?;

                            let tachyon_client = TachyonClient::new(matrix_client.clone(), config.clone(), msn_user.clone(), ticket_token.clone(), notif_sender.clone(), local_store.client_shutdown_snd.clone(), local_store.client_shutdown_recv.resubscribe());
                            let drop_guard = tachyon_state.insert_clients(ticket_token.as_str().to_owned(), tachyon_client.clone());
                            // The real client is registered under the same ticket now, so the
                            // stand-in used during device confirmation can go.
                            tachyon_state.remove_pre_session(ticket_token.as_str());

                            local_store.client_drop_guard = Some(drop_guard);
                            local_store.token = ticket_token.clone();
                            local_store.tachyon_client = Some(tachyon_client.clone());
                            local_store.matrix_client = Some(matrix_client.clone());
                            local_store.phase = ConnectionPhase::Ready;

                            sync_with_server_task(&notif_sender, local_store, &ticket_token, &matrix_client, &msn_user, tachyon_client, config, sign_in_deadline)?;
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Unsupported USR auth type during sign-in"));
                }

            }
            Ok(())
        },

        other => Err(anyhow!(
            "Unexpected command before sign-in completed: {}",
            other
        )),
    }

}

/// Produces a live matrix client for `email_addr`, walking the user through an interactive
/// login first if this instance has never authenticated that account.
async fn authenticate(
    tachyon_state: &GlobalState,
    notif_sender: &Sender<NotificationServerCommand>,
    email_addr: &EmailAddress,
    msn_user: &MsnUser,
    config: &TachyonConfig,
    deadline: Instant,
) -> Result<Client, Error> {
    let auth_use_case = tachyon_state.app_state().auth_use_case();
    let token = tachyon_state.token_for(email_addr);

    let session = match auth_use_case.restore_session(&token).await {
        Ok(restored) => {
            debug!("Restored an existing backend session for {}", email_addr.as_str());
            restored.session
        }
        Err(AuthError::BackendCredentialsNotInStore) => {
            debug!("No backend session for {}, starting interactive login", email_addr.as_str());
            interactive_login(tachyon_state, notif_sender, email_addr, msn_user, config, deadline).await?
        }
        Err(e) => return Err(anyhow!("Could not restore backend session: {:?}", e)),
    };

    // FIXME: Remove this after the refactor is done.
    let matrix_client = session
        .as_any()
        .downcast_ref::<BackendSessionMatrix>()
        .ok_or_else(|| anyhow!("Backend session is not a matrix session"))?
        .matrix_client()
        .clone();

    Ok(matrix_client)
}

/// Sends the client a `NOT` alert pointing at our web management interface and holds the sign-in open
/// until the user completes the login in their browser.
async fn interactive_login(
    tachyon_state: &GlobalState,
    notif_sender: &Sender<NotificationServerCommand>,
    email_addr: &EmailAddress,
    msn_user: &MsnUser,
    config: &TachyonConfig,
    deadline: Instant,
) -> Result<Arc<dyn tachyon_core::application::ports::BackendSession>, Error> {
    let auth_use_case = tachyon_state.app_state().auth_use_case();

    let matrix_id = email_addr.to_owned_user_id();
    let server_name = matrix_id.server_name().as_str();

    let login_start = auth_use_case
        .start_interactive_login(
            server_name,
            CoreUserId::new(matrix_id.as_str()),
            &bridge_metadata(),
        )
        .await
        .map_err(|e| anyhow!("Could not start interactive login: {:?}", e))?;

    // For OAuth the flow id is the CSRF token, which comes back as the `state` query
    // parameter, so the callback can find this login without any extra bookkeeping.
    let (flow_id, prompt_label) = match &login_start.prompt {
        InteractiveAuthStarted::OAuth { csrf_token, .. } => {
            (csrf_token.clone(), "Click here to sign in to your Matrix account.")
        }
        InteractiveAuthStarted::PasswordRequired => (
            uuid_flow_id(),
            "Click here to sign in. Your homeserver needs a password.",
        ),
    };

    let (alert, receiver) = Alert::new_interactive_login();
    tachyon_state.store_pending_login(
        flow_id.clone(),
        PendingLogin {
            login_id: login_start.login_id.clone(),
            email: email_addr.clone(),
            prompt: login_start.prompt,
            alert,
        },
    );

    let notification_id = rand::random::<i32>();
    let start_url = format!(
        "http://127.0.0.1:{}/tachyon/login/start?flow={}",
        config.http_port,
        urlencoding::encode(&flow_id)
    );

    let login_not = NotificationServerCommand::NOT(NotServer {
        payload: NotificationPayloadType::Normal(NotificationFactory::alert(
            &msn_user.uuid,
            msn_user.get_email_address(),
            prompt_label,
            format!("http://127.0.0.1:{}/tachyon", config.http_port).as_str(),
            &start_url,
            &start_url,
            Some("key-icon.gif"),
            notification_id,
        )),
    });

    notif_sender.send(login_not).await?;

    let abandon = || {
        tachyon_state.take_pending_login(&flow_id);
        tachyon_state.remove_pre_session(tachyon_state.ticket_for(email_addr).as_str());
    };

    match timeout_at(deadline, receiver.recv()).await {
        Ok(Ok(AlertSuccess::Unit)) => {}
        Ok(Ok(_)) => {
            abandon();
            return Err(anyhow!("Unexpected interactive login alert payload"));
        }
        Ok(Err(e)) => {
            abandon();
            return Err(anyhow!("Interactive login failed: {}", e));
        }
        Err(_elapsed) => {
            abandon();
            return Err(anyhow!("Interactive login was not completed in time"));
        }
    }

    let token = tachyon_state.token_for(email_addr);
    let restored = auth_use_case
        .restore_session(&token)
        .await
        .map_err(|e| anyhow!("Interactive login completed but no session was stored: {:?}", e))?;

    Ok(restored.session)
}

fn bridge_metadata() -> BridgeMetadata {
    BridgeMetadata {
        name: "Windows Live Messenger (Tachyon)".to_string(),
        client_uri: "https://tachyon.chat".to_string(),
        image_url: None,
        tos: None,
    }
}

fn uuid_flow_id() -> String {
    hex::encode(rand::random::<[u8; 16]>())
}

fn sync_with_server_task(notif_sender: &Sender<NotificationServerCommand>, local_store: &LocalClientData, ticket_token: &TicketToken, matrix_client: &Client, msn_user: &MsnUser, tachyon_client: TachyonClient, config: &TachyonConfig, deadline: Instant) -> Result<(), Error> {
    let msn_user_clone = msn_user.clone();
    let matrix_client_clone = matrix_client.clone();
    let notif_sender_clone = notif_sender.clone();
    let ticket_token_clone = ticket_token.clone();
    let config_clone = config.clone();
    let client_shutdown_snd = local_store.client_shutdown_snd.clone();
    let mut client_shutdown_recv = local_store.client_shutdown_recv.resubscribe();


    task::spawn(async move {
        let cross_signed = match check_device_is_crossed_signed(&matrix_client_clone).await {
            Ok(cross_signed) => cross_signed,
            Err(e) => {
                error!("Could not check whether the device is cross signed: {}", e);
                let _ = client_shutdown_snd.send(());
                return;
            }
        };

        debug!("Device is cross signed: {}", cross_signed);

        if !cross_signed {

            let sign_loop_kill_snd = match cross_signing::cross_sign_sync_task(&matrix_client_clone, client_shutdown_recv.resubscribe()).await {
                Ok(sender) => sender,
                Err(e) => {
                    error!("Could not start the cross signing sync task: {}", e);
                    let _ = client_shutdown_snd.send(());
                    return;
                }
            };

            let notification_id = rand::random::<i32>();

            let verif_not = NotificationServerCommand::NOT(NotServer {
                payload: NotificationPayloadType::Normal(NotificationFactory::alert(&msn_user_clone.uuid, msn_user_clone.get_email_address(), "Oops ! Your device is not verified yet ! Click here to verify.", format!("http://127.0.0.1:{}/tachyon", config_clone.http_port).as_str(), format!("http://127.0.0.1:{}/tachyon/confirm_device?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), format!("http://127.0.0.1:{}/tachyon/confirm_device?t={}", config_clone.http_port, &ticket_token_clone.as_str()).as_str(), Some("shield_verify.png"), notification_id)),
            });

            let (alert, receiver) = Alert::new_confirm_device();
            tachyon_client.alerts().insert(notification_id, alert);

            debug!("Device is not confirmed, alerting the client to verify it");
            let _ = notif_sender_clone.send(verif_not).await;

            select! {
                recv = receiver.recv() => {
                   let _ = sign_loop_kill_snd.send(()).await;
                    match recv {
                        Ok(_success) => {

                            if check_device_is_crossed_signed(&matrix_client_clone).await.unwrap_or(false) {

                            } else {
                                warn!("Device is still not cross signed after confirmation");
                                let _  = client_shutdown_snd.send(());
                                return;
                            }
                        }
                        Err(_err) => {
                            let _  = client_shutdown_snd.send(());
                            debug!("error received stopping sync_with_server_task");
                            return;
                        }
                    }
                },
                _timeout = sleep_until(deadline) => {
                    warn!("Device verification was not completed before the client gives up signing in");
                    let _ = sign_loop_kill_snd.send(()).await;
                    let _ = client_shutdown_snd.send(());
                    return;
                },
                _kill_recv = client_shutdown_recv.recv() => {
                    debug!("client_kill_recv stopping sync_with_server_task");
                    let _ = sign_loop_kill_snd.send(()).await;
                    return;
                }
            }
        }

        let _ = notif_sender_clone.send(NotificationServerCommand::RAW(RawCommand::without_payload("SBS 0 null"))).await;

        //This makes the client login to succeed and go past the loading screen.
        let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
            sender: "Hotmail".to_string(),
            display_name: DisplayName::new_from_ref("Hotmail"),
            payload: MsgPayload::Raw(RawMsgPayloadFactory::get_msmsgs_profile(
                &msn_user_clone.uuid.get_puid(),
                msn_user_clone.get_email_address(),
                &ticket_token_clone,
            )),
        });

        let _ = notif_sender_clone.send(initial_profile_msg).await;

        //Todo fetch endpoint data
        let endpoint_data = b"<Data></Data>";
        let _ = notif_sender_clone
            .send(NotificationServerCommand::RAW(RawCommand::with_payload(
                &format!("UBX 1:{}", &msn_user_clone.get_email_address().as_str()),
                endpoint_data.to_vec(),
            )))
            .await;

        //Todo check the device state before we sync

        let _sync_join_handle = sync(tachyon_client, matrix_client_clone, client_shutdown_snd, client_shutdown_recv).await;

        let initial_mail_data = NotificationServerCommand::MSG(MsgServer {
            sender: "Hotmail".to_string(),
            display_name: DisplayName::new_from_ref("Hotmail"),
            payload: MsgPayload::Raw(RawMsgPayloadFactory::get_initial_mail_data_empty_notification()),
        });

        let _ = notif_sender_clone.send(initial_mail_data).await;
    });
    Ok(())
}
