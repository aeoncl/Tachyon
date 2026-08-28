use std::sync::Arc;
use async_trait::async_trait;
use matrix_sdk::{ruma::api::client::error::ErrorKind, Client, ClientBuilder, ServerName, SessionChange};
use tokio::select;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use log::log;
use tachyon_core::application::ports::{BackendSession, ChatBackendTrait};
use tachyon_core::ids::LoginId;
use tachyon_core::application::error::BackendError;
use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::store::CredentialStore;

pub struct BackendSessionMatrix {
    client: Client,
    credentials_store: Arc<dyn CredentialStore>,
    tasks_cancellation_token: CancellationToken,
    session_tokens_watcher_task: JoinHandle<()>
}

impl BackendSessionMatrix {
    pub async fn restore(login_id: &LoginId, client: matrix_sdk::Client, session_restore_data: SessionRestoreData, credentials_store: Arc<dyn CredentialStore>) -> Result<Self, BackendError> {

        let session_cancellation_token = CancellationToken::new();


        let handle = subscribe_to_session_tokens(&login_id, &client, credentials_store.clone(), session_cancellation_token.clone())?;


        if let Err(err) = client
            .restore_session(session_restore_data)
            .await {

            session_cancellation_token.cancel();

            return Err(BackendError::CannotRestoreLogin(format!("{}", err)))
        }


        match client.whoami().await {
            Ok(_) => Ok(
                BackendSessionMatrix {
                    client,
                    credentials_store,
                    tasks_cancellation_token: session_cancellation_token.clone(),
                    session_tokens_watcher_task: handle,
                }
            ),
            Err(e) => {

                session_cancellation_token.cancel();


                let Some(api_error) = e.client_api_error_kind() else {
                    return Err(BackendError::Technical(anyhow::anyhow!(e)));
                };

                match api_error {
                    ErrorKind::Forbidden { .. } => Err(BackendError::LoggedOut),
                    ErrorKind::Unauthorized => Err(BackendError::LoggedOut),
                    ErrorKind::UnknownToken { soft_logout } => {
                        if *soft_logout {
                            Err(BackendError::SoftLoggedOut)
                        } else {
                            Err(BackendError::LoggedOut)
                        }
                    }
                    _ => Err(BackendError::Technical(anyhow::anyhow!(e))),
                }
            }
        }
    }
}

impl BackendSession for BackendSessionMatrix {


}

pub struct ChatBackendMatrix {
    credentials_store: Arc<dyn CredentialStore>,
}

#[async_trait]
impl ChatBackendTrait for ChatBackendMatrix {
    async fn restore_login(
        &self,
        login_id: LoginId,
    ) -> Result<Arc<dyn BackendSession>, BackendError> {

        let Some(session_restore_data) = self
            .credentials_store
            .session_restore_data_by_login_id(&login_id)
            .await?
        else {
            return Err(BackendError::LoggedOut);
        };

        let server_name = session_restore_data.user_id.server_name();

        //Todo move those properties to BackendConfig
        //Todo also configure store when building the client
        let client = matrix_client_builder(server_name, None, false)
            .build()
            .await
            .map_err(|e| BackendError::Technical(anyhow::anyhow!(e)))?;

        let session = BackendSessionMatrix::restore(&login_id, client, session_restore_data, self.credentials_store.clone()).await?;

        Ok(Arc::new(session))
    }
}

fn matrix_client_builder(
    server_name: &ServerName,
    homeserver_url: Option<String>,
    disable_ssl: bool,
) -> ClientBuilder {
    let mut client_builder = Client::builder();

    client_builder = client_builder.handle_refresh_tokens();

    if disable_ssl {
        client_builder = client_builder.disable_ssl_verification();
    }

    match homeserver_url {
        None => client_builder = client_builder.server_name(server_name),
        Some(homeserver_url) => client_builder = client_builder.homeserver_url(&homeserver_url),
    }

    client_builder
}

fn subscribe_to_session_tokens(login_id: &LoginId, client: &matrix_sdk::Client, credentials_store: Arc<dyn CredentialStore>, cancellation_token: CancellationToken) -> Result<JoinHandle<()>, BackendError> {
    let mut receiver = client.subscribe_to_session_changes();

    let login_id_clone = login_id.clone();
    let client_clone = client.clone();
    let handle = tokio::spawn(async move {
        loop {

            select! {

                cancel = cancellation_token.cancelled() => {
                    break;
                }

                session_change = receiver.recv() => {

                    let Ok(session_change) = session_change else {
                        break;
                    };

                    match session_change {
                        SessionChange::UnknownToken { soft_logout } => {
                            //Todo push Logout or SoftLogoutEvent
                        }
                        SessionChange::TokensRefreshed => {
                            if let Some(session_tokens) = client_clone.session_tokens() {
                                let _ = credentials_store.update_tokens(&login_id_clone, session_tokens.access_token, session_tokens.refresh_token).await;
                            }
                        }
                    }

                }
            }
        }
    });

    Ok(handle)
}