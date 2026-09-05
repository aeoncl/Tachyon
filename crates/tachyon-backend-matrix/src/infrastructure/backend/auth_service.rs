use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc};
use std::time::{Duration, Instant};
use anyhow::anyhow;
use async_trait::async_trait;
use matrix_sdk::{Client, ServerName, SessionChange};
use matrix_sdk::authentication::oauth::registration::{ApplicationType, ClientMetadata, Localized, OAuthGrantType};
use matrix_sdk::authentication::oauth::UrlOrQuery;
use matrix_sdk::reqwest::Url;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::serde::Raw;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tachyon_core::application::error::BackendError;
use tachyon_core::application::ports::{AuthService, BackendSession, CredentialRepository};
use tachyon_core::domain::auth::{BridgeMetadata, InteractiveAuthStarted};
use tachyon_core::domain::ids::{LoginId, UserId};
use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::backend::session::BackendSessionMatrix;
use crate::infrastructure::mappers::IntoMapper;

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

#[derive(Clone, Default)]
pub struct MatrixBackendConfig {
    pub store_root: Option<PathBuf>,
    pub disable_ssl: bool,
    pub homeserver_url_override: Option<String>,
}

pub struct AuthServiceMatrixSdk {
    credential_repository: Arc<dyn CredentialRepository>,
    pending_clients: Mutex<HashMap<String, PendingClient>>,
    config: MatrixBackendConfig,
}

impl AuthServiceMatrixSdk {
    pub fn new(
        credential_repository: Arc<dyn CredentialRepository>,
        config: MatrixBackendConfig,
    ) -> Self {
        Self {
            credential_repository,
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

        // Do we really need to create the store here ? doesn't the SDK makes sure the folder exist ? If not, we should move that to a different service cause that's unrelated IO.
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

    async fn store_credentials(
        &self,
        login_id: &LoginId,
        client: &Client,
    ) -> Result<(), BackendError> {
        let session = client
            .session()
            .ok_or_else(|| BackendError::Technical(anyhow!("Client has no session after login")))?;

        let blob = SessionRestoreData::try_from(session)
            .map_err(BackendError::Technical)?
            .to_blob()
            .map_err(BackendError::Technical)?;

        self.credential_repository.store(login_id, blob).await?;

        Ok(())
    }
}

#[async_trait]
impl AuthService for AuthServiceMatrixSdk {
    async fn restore_session(
        &self,
        login_id: LoginId,
    ) -> Result<Arc<dyn BackendSession>, BackendError> {
        let Some(blob) = self.credential_repository.credentials(&login_id).await? else {
            return Err(BackendError::LoggedOut);
        };

        let session_restore_data =
            SessionRestoreData::from_blob(&blob).map_err(BackendError::Technical)?;

        let user_id = session_restore_data.user_id.clone();
        let client = self
            .build_client(user_id.server_name(), Some(&user_id))
            .await?;

        let session_cancellation_token = CancellationToken::new();

        let _handle = subscribe_to_session_tokens(
            &login_id,
            &client,
            self.credential_repository.clone(),
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


        {
            let mut pending_clients = self.pending_clients
                .lock()
                .await;

            pending_clients.retain(|_, v| !v.is_expired());
        }

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

        {
            let mut pending_clients = self.pending_clients
                .lock()
                .await;

            pending_clients
                .insert(login_id.to_string(), PendingClient::new(client.clone()));

        }


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

            let found = pending
                .remove(&login_id.to_string())
                .map(|p| p.client)
                .ok_or(BackendError::LoggedOut)?;

            // Purge expired entries
            pending.retain(|_, v| !v.is_expired());

            found
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
            self.credential_repository.clone(),
            session_cancellation_token.clone(),
        )?;

        let session = BackendSessionMatrix::new(client, session_cancellation_token);

        Ok(Arc::new(session))
    }
}

fn sanitize_user_id(user_id: &matrix_sdk::ruma::UserId) -> String {
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, user_id.as_str().as_bytes())
        .to_string()
        .to_uppercase()
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
    credential_repository: Arc<dyn CredentialRepository>,
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

                    let session_change = match session_change {
                        Ok(session_change) => session_change,
                        Err(RecvError::Lagged(_)) => continue,
                        Err(RecvError::Closed) => break,
                    };

                    match session_change {
                        SessionChange::UnknownToken { soft_logout: _ } => {
                            //Todo push Logout or SoftLogoutEvent
                        }
                        SessionChange::TokensRefreshed => {
                            persist_refreshed_tokens(&login_id_clone, &client_clone, &credential_repository).await;
                        }
                    }

                }
            }
        }
    });

    Ok(handle)
}

async fn persist_refreshed_tokens(
    login_id: &LoginId,
    client: &Client,
    credential_repository: &Arc<dyn CredentialRepository>,
) {
    let Some(session) = client.session() else {
        log::warn!("Tokens refreshed but the client has no session to persist");
        return;
    };

    let blob = SessionRestoreData::try_from(session).and_then(|data| data.to_blob());

    match blob {
        Ok(blob) => {
            if let Err(e) = credential_repository.store(login_id, blob).await {
                log::warn!("Could not persist refreshed tokens: {:?}", e);
            }
        }
        Err(e) => log::warn!("Could not serialize refreshed tokens: {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tachyon_testkit::repositories::CredentialRepositoryInMem;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn build_test_client() -> (Client, MockServer) {
        let mock_server = MockServer::start().await;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200).set_body_string("{}"))
            .mount(&mock_server)
            .await;

        let auth_service = AuthServiceMatrixSdk::new(
            Arc::new(CredentialRepositoryInMem::default()),
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
    fn sanitized_user_id_matches_legacy_store_directory_scheme() {
        let user_id = matrix_sdk::ruma::UserId::parse("@aeon:shlasouf.local").unwrap();
        let sanitized = sanitize_user_id(&user_id);

        // Pinned value: upper-cased UUID v5 (OID namespace) of "@aeon:shlasouf.local".
        assert_eq!(sanitized, "264E4340-A168-537C-890B-946D4EB046E0");
    }

    #[tokio::test]
    async fn stored_credentials_round_trip_through_the_repository() {
        let (client, _mock) = build_test_client().await;
        let restore_data = SessionRestoreData {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            user_id: matrix_sdk::ruma::UserId::parse("@aeon:shlasouf.local").unwrap().to_owned(),
            device_id: matrix_sdk::ruma::OwnedDeviceId::from("DEVICEID"),
            auth_kind: crate::domain::auth::AuthKind::Matrix,
        };
        client.restore_session(restore_data.clone()).await.unwrap();

        let repository = Arc::new(CredentialRepositoryInMem::default());
        let auth_service = AuthServiceMatrixSdk::new(
            repository.clone(),
            MatrixBackendConfig::default(),
        );
        let login_id = LoginId::new("l1");

        auth_service.store_credentials(&login_id, &client).await.unwrap();

        let blob = repository.credentials(&login_id).await.unwrap().unwrap();
        let restored = SessionRestoreData::from_blob(&blob).unwrap();
        assert!(restored == restore_data);
    }
}