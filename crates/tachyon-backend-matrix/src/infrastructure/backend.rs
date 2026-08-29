use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::mappers::IntoMapper;
use crate::infrastructure::repositories::CredentialsRepository;
use anyhow::anyhow;
use async_trait::async_trait;
use matrix_sdk::authentication::oauth::registration::{ApplicationType, ClientMetadata, Localized, OAuthGrantType};
use matrix_sdk::reqwest::Url;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{ruma::api::client::error::ErrorKind, Client, ClientBuilder, ServerName, SessionChange};
use std::sync::Arc;
use tachyon_core::application::error::BackendError;
use tachyon_core::application::ports::BackendSession;
use tachyon_core::domain::auth::InteractiveAuthStarted;
use tachyon_core::domain::backend_ports::AuthService;
use tachyon_core::domain::ids::{LoginId, UserId};
use tokio::select;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct BackendSessionMatrix {
    client: Client,
    tasks_cancellation_token: CancellationToken
}

impl BackendSessionMatrix {

    /* You need to subscribe to Session Token change before restoring Auth */
    pub async fn restore(client: matrix_sdk::Client, cancellation_token: CancellationToken, session_restore_data: SessionRestoreData) -> Result<Self, BackendError> {

        if let Err(err) = client
            .restore_session(session_restore_data)
            .await {

            cancellation_token.cancel();

            return Err(BackendError::CannotRestoreLogin(format!("{}", err)))
        }

        match client.whoami().await {
            Ok(_) => Ok(
                BackendSessionMatrix {
                    client,
                    tasks_cancellation_token: cancellation_token.clone(),
                }
            ),
            Err(e) => {

                cancellation_token.cancel();


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

    pub fn pending_auth(client: matrix_sdk::Client, cancellation_token: CancellationToken) -> Result<Self, BackendError> {
        Ok(BackendSessionMatrix {
            client,
            tasks_cancellation_token: cancellation_token.clone(),
        })
    }
}

impl BackendSession for BackendSessionMatrix {




}

pub struct AuthServiceMatrixSdk {
    credentials_store: Arc<dyn CredentialsRepository>,
}

#[async_trait]
impl AuthService for AuthServiceMatrixSdk {
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
            .map_err(|e| BackendError::Technical(anyhow::anyhow!("{}", e)))?;

        let session_cancellation_token = CancellationToken::new();

        let handle = subscribe_to_session_tokens(&login_id, &client, self.credentials_store.clone(), session_cancellation_token.clone())?;
        let session = BackendSessionMatrix::restore(client, session_cancellation_token.clone(), session_restore_data,).await?;

        Ok(Arc::new(session))
    }

    async fn start_interactive_login(&self, login_id: &LoginId, server_name: &str, user_id: Option<UserId>, redirect_url: &str) -> Result<InteractiveAuthStarted, BackendError> {

        let user_id: Option<OwnedUserId> = match user_id {
            None => None,
            Some(user_id) => Some(user_id.map_into().map_err(|e| BackendError::Technical(anyhow!("{:?}", e)))?)
        };

        let server_name = ServerName::parse(server_name).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        let client = matrix_client_builder(&server_name, None, false)
            .build()
            .await
            .map_err(|e| BackendError::Technical(anyhow::anyhow!(e)))?;

       let _ = client.oauth().cached_server_metadata().await.map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;
       let redirect_url = Url::parse(redirect_url).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;
       let client_metadata = ClientMetadata::new(ApplicationType::Native, vec![OAuthGrantType::AuthorizationCode { redirect_uris: vec![redirect_url.clone()] }], Localized::new(Url::parse("https://tachyon.chat").unwrap(), vec![]));
       let raw_client_metadata = Raw::new(&client_metadata).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;
       let _ = client.oauth().register_client(&raw_client_metadata).await.map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

       let authorization_data = {
           let mut builder = client.oauth().login(redirect_url, None, None, None);
           if let Some(user_id) = user_id {
               builder = builder.user_id_hint(&user_id);
           }
           builder.build().await.map_err(|e| BackendError::Technical(anyhow!("{}", e)))?
       };

        let session_cancellation_token = CancellationToken::new();
        let backend_session = BackendSessionMatrix::pending_auth(client, session_cancellation_token)?;

       Ok(InteractiveAuthStarted {
           backend_session: Arc::new(backend_session),
           auth_url: authorization_data.url.to_string(),
           csrf_token: authorization_data.state.into_secret()
       })

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

fn subscribe_to_session_tokens(login_id: &LoginId, client: &matrix_sdk::Client, credentials_store: Arc<dyn CredentialsRepository>, cancellation_token: CancellationToken) -> Result<JoinHandle<()>, BackendError> {

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