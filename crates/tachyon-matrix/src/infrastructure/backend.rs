use std::sync::Arc;

use async_trait::async_trait;
use matrix_sdk::{Client, ClientBuilder, ServerName, ruma::api::client::error::ErrorKind};
use tachyon_core::{
    domain::ids::LoginId,
    port::{
        backend::{BackendRestoreOutcome, BackendSession, ChatBackend},
        error::BackendError::{self, Technical},
    },
};

use crate::infrastructure::store::CredentialStore;

pub struct BackendSessionMatrix {
    client: Client,
}

impl BackendSessionMatrix {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

impl BackendSession for BackendSessionMatrix {}

pub struct ChatBackendMatrix {
    credentials_store: Arc<dyn CredentialStore>,
}

#[async_trait]
impl ChatBackend for ChatBackendMatrix {
    async fn restore_login(
        &self,
        login_id: LoginId,
    ) -> Result<BackendRestoreOutcome, BackendError> {
        let Some(session_restore_data) = self
            .credentials_store
            .session_restore_data_by_login_id(&login_id)
            .await?
        else {
            return Ok(BackendRestoreOutcome::LoggedOut);
        };

        let server_name = session_restore_data.user_id.server_name();

        //Todo move those properties to BackendConfig
        //Todo also configure store when building the client
        let client = matrix_client_builder(server_name, None, false)
            .build()
            .await
            .map_err(|e| BackendError::Technical(anyhow::anyhow!(e)))?;

        //Todo Start a task to subscribe to session changes and fetch and store the refresh tokens before we restore session.

        client
            .restore_session(session_restore_data)
            .await
            .map_err(|e| BackendError::CannotRestoreLogin(format!("{}", e)))?;

        match client.whoami().await {
            Ok(_) => Ok(BackendRestoreOutcome::Success(Arc::new(
                BackendSessionMatrix::new(client),
            ))),
            Err(e) => {
                let Some(api_error) = e.client_api_error_kind() else {
                    return Err(BackendError::Technical(anyhow::anyhow!(e)));
                };

                match api_error {
                    ErrorKind::Forbidden { .. } => Ok(BackendRestoreOutcome::LoggedOut),
                    ErrorKind::Unauthorized => Ok(BackendRestoreOutcome::LoggedOut),
                    ErrorKind::UnknownToken { soft_logout } => {
                        if *soft_logout {
                            Ok(BackendRestoreOutcome::SoftLoggedOut)
                        } else {
                            Ok(BackendRestoreOutcome::LoggedOut)
                        }
                    }
                    _ => Err(Technical(anyhow::anyhow!(e))),
                }
            }
        }
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
