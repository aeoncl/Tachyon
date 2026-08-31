use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::mappers::IntoMapper;
use crate::infrastructure::repositories::CredentialsRepository;
use anyhow::anyhow;
use async_trait::async_trait;
use matrix_sdk::authentication::oauth::registration::{
    ApplicationType, ClientMetadata, Localized, OAuthGrantType,
};
use matrix_sdk::authentication::oauth::UrlOrQuery;
use matrix_sdk::reqwest::Url;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{
    ruma::api::client::error::ErrorKind, Client, ServerName, SessionChange,
};
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tachyon_core::application::error::BackendError;
use tachyon_core::application::ports::{AuthService, BackendSession};
use tachyon_core::domain::auth::{BridgeMetadata, InteractiveAuthStarted};
use tachyon_core::domain::ids::{LoginId, UserId};
use tokio::select;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

/// How the adapter builds matrix clients.
#[derive(Clone, Default)]
pub struct MatrixBackendConfig {
    /// Root under which each account gets its own SQLite store. Without one the client is
    /// memory-only, so crypto identity — and therefore device verification — is lost on
    /// every restart.
    pub store_root: Option<PathBuf>,
    pub disable_ssl: bool,
    /// Bypasses server-name discovery. Test and development use only.
    pub homeserver_url_override: Option<String>,
}

pub struct BackendSessionMatrix {
    client: Client,
    tasks_cancellation_token: CancellationToken,
}

impl BackendSessionMatrix {
    /* You need to subscribe to Session Token change before restoring Auth */
    pub async fn restore(
        client: matrix_sdk::Client,
        cancellation_token: CancellationToken,
        session_restore_data: SessionRestoreData,
    ) -> Result<Self, BackendError> {
        if let Err(err) = client.restore_session(session_restore_data).await {
            cancellation_token.cancel();

            return Err(BackendError::CannotRestoreLogin(format!("{}", err)));
        }

        match client.whoami().await {
            Ok(_) => Ok(BackendSessionMatrix {
                client,
                tasks_cancellation_token: cancellation_token.clone(),
            }),
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

    /// TEMPORARY (refactor scaffold): the legacy `tachyon` crate still drives matrix-sdk
    /// directly. Reached through [`BackendSession::as_any`]; goes away with it.
    pub fn matrix_client(&self) -> &Client {
        &self.client
    }
}

impl BackendSession for BackendSessionMatrix {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for BackendSessionMatrix {
    fn drop(&mut self) {
        // Otherwise the token-refresh subscriber outlives the session it belongs to.
        self.tasks_cancellation_token.cancel();
    }
}

const PENDING_CLIENT_TTL: Duration = Duration::from_secs(600);

struct PendingClient {
    client: Client,
    created_at: Instant,
}

impl PendingClient {
    fn new(client: Client) -> Self {
        Self {
            client,
            created_at: Instant::now(),
        }
    }

    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > PENDING_CLIENT_TTL
    }
}

pub struct AuthServiceMatrixSdk {
    credentials_store: Arc<dyn CredentialsRepository>,
    pending_clients: Mutex<HashMap<String, PendingClient>>,
    config: MatrixBackendConfig,
}

impl AuthServiceMatrixSdk {
    pub fn new(
        credentials_store: Arc<dyn CredentialsRepository>,
        config: MatrixBackendConfig,
    ) -> Self {
        Self {
            credentials_store,
            pending_clients: Mutex::new(HashMap::new()),
            config,
        }
    }

    async fn build_client(
        &self,
        server_name: &ServerName,
        user_id: Option<&matrix_sdk::ruma::UserId>,
    ) -> Result<Client, BackendError> {
        let mut client_builder = Client::builder().handle_refresh_tokens();

        if self.config.disable_ssl {
            client_builder = client_builder.disable_ssl_verification();
        }

        match &self.config.homeserver_url_override {
            None => client_builder = client_builder.server_name(server_name),
            Some(homeserver_url) => client_builder = client_builder.homeserver_url(homeserver_url),
        }

        // The store is per-account, so it can only be attached once the user is known. An
        // interactive login without a user id hint therefore starts memory-only.
        if let (Some(store_root), Some(user_id)) = (&self.config.store_root, user_id) {
            let store_path = store_root.join(sanitize_user_id(user_id)).join("store");
            std::fs::create_dir_all(&store_path).map_err(|e| {
                BackendError::Technical(anyhow!("Could not create store dir: {}", e))
            })?;
            client_builder = client_builder.sqlite_store(store_path, None);
        }

        client_builder
            .build()
            .await
            .map_err(|e| BackendError::Technical(anyhow::anyhow!("{}", e)))
    }

    /// Persist what the SDK ended up with, so the login can be restored later.
    async fn store_credentials(
        &self,
        login_id: &LoginId,
        client: &Client,
    ) -> Result<(), BackendError> {
        let session = client
            .session()
            .ok_or_else(|| BackendError::Technical(anyhow!("Client has no session after login")))?;

        let restore_data = SessionRestoreData::try_from(session)
            .map_err(BackendError::Technical)?;

        self.credentials_store.insert(login_id, restore_data).await?;

        Ok(())
    }
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

        let user_id = session_restore_data.user_id.clone();
        let client = self
            .build_client(user_id.server_name(), Some(&user_id))
            .await?;

        let session_cancellation_token = CancellationToken::new();

        let _handle = subscribe_to_session_tokens(
            &login_id,
            &client,
            self.credentials_store.clone(),
            session_cancellation_token.clone(),
        )?;

        let session = BackendSessionMatrix::restore(
            client,
            session_cancellation_token.clone(),
            session_restore_data,
        )
        .await?;

        Ok(Arc::new(session))
    }

    async fn start_interactive_login(
        &self,
        login_id: &LoginId,
        server_name: &str,
        user_id: Option<UserId>,
        redirect_url: &str,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<InteractiveAuthStarted, BackendError> {
        let user_id: Option<OwnedUserId> = match user_id {
            None => None,
            Some(user_id) => Some(
                user_id
                    .map_into()
                    .map_err(|e| BackendError::Technical(anyhow!("{:?}", e)))?,
            ),
        };

        let server_name =
            ServerName::parse(server_name).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        let client = self.build_client(&server_name, user_id.as_deref()).await?;

        // No OAuth on this homeserver: the bridge has to collect a password itself.
        if client.oauth().cached_server_metadata().await.is_err() {
            return Ok(InteractiveAuthStarted::PasswordRequired);
        }

        self.pending_clients
            .lock()
            .await
            .insert(login_id.to_string(), PendingClient::new(client.clone()));

        let redirect_url =
            Url::parse(redirect_url).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;
        let client_metadata = build_client_metadata(bridge_metadata, redirect_url.clone())?;
        let raw_client_metadata =
            Raw::new(&client_metadata).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;
        let _ = client
            .oauth()
            .register_client(&raw_client_metadata)
            .await
            .map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        let authorization_data = {
            let mut builder = client.oauth().login(redirect_url, None, None, None);
            if let Some(user_id) = user_id {
                builder = builder.user_id_hint(&user_id);
            }
            builder
                .build()
                .await
                .map_err(|e| BackendError::Technical(anyhow!("{}", e)))?
        };

        Ok(InteractiveAuthStarted::OAuth {
            auth_url: authorization_data.url.to_string(),
            csrf_token: authorization_data.state.into_secret(),
        })
    }

    async fn finish_interactive_login(
        &self,
        login_id: &LoginId,
        callback_query: &str,
    ) -> Result<Arc<dyn BackendSession>, BackendError> {
        let client = {
            let mut pending = self.pending_clients.lock().await;

            // Purge expired entries
            pending.retain(|_, v| !v.is_expired());

            pending
                .remove(&login_id.to_string())
                .map(|p| p.client)
                .ok_or(BackendError::LoggedOut)?
        };

        client
            .oauth()
            .finish_login(UrlOrQuery::Query(callback_query.to_string()))
            .await
            .map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        self.store_credentials(login_id, &client).await?;

        let session_cancellation_token = CancellationToken::new();
        let _handle = subscribe_to_session_tokens(
            login_id,
            &client,
            self.credentials_store.clone(),
            session_cancellation_token.clone(),
        )?;

        let session = BackendSessionMatrix {
            client,
            tasks_cancellation_token: session_cancellation_token,
        };

        Ok(Arc::new(session))
    }
}

/// Matrix user ids contain characters that are not legal in a Windows path component.
fn sanitize_user_id(user_id: &matrix_sdk::ruma::UserId) -> String {
    user_id
        .as_str()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn build_client_metadata(
    bridge_metadata: &BridgeMetadata,
    redirect_url: Url,
) -> Result<ClientMetadata, BackendError> {
    let client_uri = Url::parse(&bridge_metadata.client_uri)
        .map_err(|e| BackendError::Technical(anyhow!("Invalid bridge client_uri: {}", e)))?;

    let mut metadata = ClientMetadata::new(
        ApplicationType::Native,
        vec![OAuthGrantType::AuthorizationCode {
            redirect_uris: vec![redirect_url],
        }],
        Localized::new(client_uri, vec![]),
    );

    metadata.client_name = Some(Localized::new(bridge_metadata.name.clone(), vec![]));

    if let Some(image_url) = &bridge_metadata.image_url {
        let image_url = Url::parse(image_url)
            .map_err(|e| BackendError::Technical(anyhow!("Invalid bridge image_url: {}", e)))?;
        metadata.logo_uri = Some(Localized::new(image_url, vec![]));
    }

    if let Some(tos) = &bridge_metadata.tos {
        let tos = Url::parse(tos)
            .map_err(|e| BackendError::Technical(anyhow!("Invalid bridge tos url: {}", e)))?;
        metadata.tos_uri = Some(Localized::new(tos, vec![]));
    }

    Ok(metadata)
}

fn subscribe_to_session_tokens(
    login_id: &LoginId,
    client: &matrix_sdk::Client,
    credentials_store: Arc<dyn CredentialsRepository>,
    cancellation_token: CancellationToken,
) -> Result<JoinHandle<()>, BackendError> {
    let mut receiver = client.subscribe_to_session_changes();

    let login_id_clone = login_id.clone();
    let client_clone = client.clone();
    let handle = tokio::spawn(async move {
        loop {
            select! {

                _cancel = cancellation_token.cancelled() => {
                    break;
                }

                session_change = receiver.recv() => {

                    let Ok(session_change) = session_change else {
                        break;
                    };

                    match session_change {
                        SessionChange::UnknownToken { soft_logout: _ } => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::CredentialsRepositoryInMem;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Builds a matrix `Client` against a wiremock server so that the SDK's
    /// internal capability-detection request during `.build()` succeeds without
    /// needing a real homeserver.
    async fn build_test_client() -> (Client, MockServer) {
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&mock_server)
            .await;

        let auth_service = AuthServiceMatrixSdk::new(
            Arc::new(CredentialsRepositoryInMem::default()),
            MatrixBackendConfig {
                store_root: None,
                disable_ssl: false,
                homeserver_url_override: Some(mock_server.uri()),
            },
        );

        let client = auth_service
            .build_client(&ServerName::parse("localhost").unwrap(), None)
            .await
            .unwrap();

        (client, mock_server)
    }

    #[tokio::test]
    async fn test_pending_client_not_expired_when_fresh() {
        let (client, _mock) = build_test_client().await;

        let pc = PendingClient::new(client);
        assert!(!pc.is_expired(), "freshly created client should not be expired");
    }

    #[tokio::test]
    async fn test_pending_client_expired_after_ttl() {
        let (client, _mock) = build_test_client().await;

        let pc = PendingClient {
            client,
            created_at: Instant::now() - PENDING_CLIENT_TTL - Duration::from_secs(1),
        };
        assert!(pc.is_expired(), "client should be expired after TTL has elapsed");
    }

    #[test]
    fn sanitized_user_id_is_a_legal_path_component() {
        let user_id = matrix_sdk::ruma::UserId::parse("@aeon:shlasouf.local").unwrap();
        let sanitized = sanitize_user_id(&user_id);

        assert_eq!(sanitized, "_aeon_shlasouf.local");
    }
}
