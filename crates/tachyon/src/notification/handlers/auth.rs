use crate::matrix::sync::sync;
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use anyhow::{anyhow, Error};
use log::{debug, error};
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
use tachyon_backend_matrix::infrastructure::backend::session::BackendSessionMatrix;
use tachyon_core::application::auth_use_case::{AuthUseCase, SignIn};
use tachyon_core::application::logins::{Attempt, Step};
use tachyon_core::application::ports::BackendSession;
use tachyon_core::domain::auth::{BridgeMetadata, InteractiveAuthStarted, TachyonToken};
use tachyon_core::domain::ids::UserId as CoreUserId;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use tokio::time::{timeout_at, Instant};
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

                            let (matrix_client, attempt) = authenticate(
                                tachyon_state,
                                &notif_sender,
                                &email_addr,
                                &msn_user,
                                config,
                                sign_in_deadline,
                                local_store.client_shutdown_recv.resubscribe(),
                            ).await?;

                            let tachyon_client = TachyonClient::new(matrix_client.clone(), config.clone(), msn_user.clone(), ticket_token.clone(), notif_sender.clone(), local_store.client_shutdown_snd.clone(), local_store.client_shutdown_recv.resubscribe());
                            let drop_guard = tachyon_state.insert_clients(ticket_token.as_str().to_owned(), tachyon_client.clone(), attempt);

                            local_store.client_drop_guard = Some(drop_guard);
                            local_store.token = ticket_token.clone();
                            local_store.tachyon_client = Some(tachyon_client.clone());
                            local_store.matrix_client = Some(matrix_client.clone());
                            local_store.phase = ConnectionPhase::Ready;

                            sync_with_server_task(&notif_sender, local_store, &ticket_token, &matrix_client, &msn_user, tachyon_client)?;
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

/// Produces a live matrix client for `email_addr`, walking the user through the browser
/// steps first when this instance has never authenticated the account or does not trust
/// this device yet. Any failure past the sign-in drops the login: a backend session does
/// not outlive the Messenger connection it was opened for. The attempt names that login
/// for the connection's drop guard.
async fn authenticate(
    tachyon_state: &GlobalState,
    notif_sender: &Sender<NotificationServerCommand>,
    email_addr: &EmailAddress,
    msn_user: &MsnUser,
    config: &TachyonConfig,
    deadline: Instant,
    client_shutdown_recv: broadcast::Receiver<()>,
) -> Result<(Client, Attempt), Error> {
    let auth_use_case = tachyon_state.app_state().auth_use_case();
    let token = tachyon_state.token_for(email_addr);
    let matrix_id = email_addr.to_owned_user_id();

    let signed_in = auth_use_case
        .sign_in(
            &token,
            matrix_id.server_name().as_str(),
            CoreUserId::new(matrix_id.as_str()),
            &bridge_metadata(),
        )
        .await
        .map_err(|e| anyhow!("Could not sign in: {:?}", e))?;

    let (session, attempt) = match signed_in {
        SignIn::Ready { session, attempt } => (session, attempt),
        SignIn::Pending { step, url, attempt } => {
            debug!("Sign-in for {} needs the browser: {:?}", email_addr.as_str(), step);
            let waited = wait_for_browser(
                auth_use_case,
                notif_sender,
                &token,
                &step,
                &url,
                msn_user,
                config,
                deadline,
                client_shutdown_recv,
            )
            .await;
            match waited {
                Ok(session) => (session, attempt),
                Err(e) => {
                    give_up(auth_use_case, &token, attempt).await;
                    return Err(e);
                }
            }
        }
    };

    // FIXME: Remove this after the refactor is done.
    match session.as_any().downcast_ref::<BackendSessionMatrix>() {
        Some(matrix) => Ok((matrix.matrix_client().clone(), attempt)),
        None => {
            give_up(auth_use_case, &token, attempt).await;
            Err(anyhow!("Backend session is not a matrix session"))
        }
    }
}

async fn wait_for_browser(
    auth_use_case: &AuthUseCase,
    notif_sender: &Sender<NotificationServerCommand>,
    token: &TachyonToken,
    step: &Step,
    url: &str,
    msn_user: &MsnUser,
    config: &TachyonConfig,
    deadline: Instant,
    mut client_shutdown_recv: broadcast::Receiver<()>,
) -> Result<Arc<dyn BackendSession>, Error> {
    notif_sender
        .send(browser_step_alert(step, url, msn_user, config))
        .await?;

    select! {
        waited = timeout_at(deadline, auth_use_case.wait_for_session(token)) => match waited {
            Ok(Ok(session)) => Ok(session),
            Ok(Err(e)) => Err(anyhow!("The sign-in could not be completed: {:?}", e)),
            Err(_elapsed) => Err(anyhow!("The sign-in was not completed in time")),
        },
        _shutdown = client_shutdown_recv.recv() => {
            Err(anyhow!("The client disconnected while signing in"))
        }
    }
}

async fn give_up(auth_use_case: &AuthUseCase, token: &TachyonToken, attempt: Attempt) {
    if let Err(e) = auth_use_case.abandon(token, attempt).await {
        error!("Could not abandon the sign-in: {:?}", e);
    }
}

/// The `NOT` alert that sends the user to `url`, the page for the step the sign-in is
/// parked on.
fn browser_step_alert(
    step: &Step,
    url: &str,
    msn_user: &MsnUser,
    config: &TachyonConfig,
) -> NotificationServerCommand {
    let (label, icon) = match step {
        Step::Authenticate { prompt, .. } => (
            match prompt {
                InteractiveAuthStarted::OAuth { .. } => {
                    "Click here to sign in to your Matrix account."
                }
                InteractiveAuthStarted::PasswordRequired => {
                    "Click here to sign in. Your homeserver needs a password."
                }
            },
            "key-icon.gif",
        ),
        Step::VerifyDevice => (
            "Oops ! Your device is not verified yet ! Click here to verify.",
            "shield_verify.png",
        ),
    };

    NotificationServerCommand::NOT(NotServer {
        payload: NotificationPayloadType::Normal(NotificationFactory::alert(
            &msn_user.uuid,
            msn_user.get_email_address(),
            label,
            format!("http://127.0.0.1:{}/tachyon", config.http_port).as_str(),
            url,
            url,
            Some(icon),
            rand::random::<i32>(),
        )),
    })
}

fn bridge_metadata() -> BridgeMetadata {
    BridgeMetadata {
        name: "Windows Live Messenger (Tachyon)".to_string(),
        client_uri: "https://tachyon.chat".to_string(),
        image_url: None,
        tos: None,
    }
}

fn sync_with_server_task(notif_sender: &Sender<NotificationServerCommand>, local_store: &LocalClientData, ticket_token: &TicketToken, matrix_client: &Client, msn_user: &MsnUser, tachyon_client: TachyonClient) -> Result<(), Error> {
    let msn_user_clone = msn_user.clone();
    let matrix_client_clone = matrix_client.clone();
    let notif_sender_clone = notif_sender.clone();
    let ticket_token_clone = ticket_token.clone();
    let client_shutdown_snd = local_store.client_shutdown_snd.clone();
    let client_shutdown_recv = local_store.client_shutdown_recv.resubscribe();


    task::spawn(async move {
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
