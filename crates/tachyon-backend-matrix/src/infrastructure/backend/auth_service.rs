use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use matrix_sdk::authentication::oauth::registration::{
    ApplicationType, ClientMetadata, Localized, OAuthGrantType,
};
use matrix_sdk::reqwest::Url;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::api::client::error::ErrorKind;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::{Client, HttpError, ServerName};

use tachyon_core::application::error::BackendError;
use tachyon_core::application::ports::{AuthService, BackendSession, CredentialRepository};
use tachyon_core::domain::auth::{BridgeMetadata, InteractiveAuthStarted};
use tachyon_core::domain::ids::{LoginId, UserId};

use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::backend::session::BackendSessionMatrix;
use crate::infrastructure::mappers::IntoMapper;

#[derive(Clone, Default)]
pub struct MatrixBackendConfig {
    pub store_root: Option<PathBuf>,
    pub disable_ssl: bool,
    pub homeserver_url_override: Option<String>,
}

pub struct AuthServiceMatrixSdk {
    credential_repository: Arc<dyn CredentialRepository>,
    config: MatrixBackendConfig,
}

impl AuthServiceMatrixSdk {
    pub fn new(
        credential_repository: Arc<dyn CredentialRepository>,
        config: MatrixBackendConfig,
    ) -> Self {
        Self {
            credential_repository,
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
}

#[async_trait]
impl AuthService for AuthServiceMatrixSdk {
    async fn restore(&self, login_id: &LoginId) -> Result<Arc<dyn BackendSession>, BackendError> {
        let Some(blob) = self.credential_repository.credentials(login_id).await? else {
            return Err(BackendError::LoggedOut);
        };

        let session_restore_data =
            SessionRestoreData::from_blob(&blob).map_err(BackendError::Technical)?;

        let user_id = session_restore_data.user_id.clone();
        let client = self
            .build_client(user_id.server_name(), Some(&user_id))
            .await?;

        let session = BackendSessionMatrix::new(
            client,
            login_id.clone(),
            self.credential_repository.clone(),
        );
        // The first request after restore can already refresh the tokens, and MAS rotates
        // the refresh token with them. The watcher has to be listening before that or the
        // rotated token is never persisted and the next restart is logged out.
        session.spawn_token_watcher();

        if let Err(err) = session.matrix_client().restore_session(session_restore_data).await {
            session.close().await;
            return Err(BackendError::CannotRestoreLogin(format!("{}", err)));
        }

        if let Err(err) = session.matrix_client().whoami().await {
            session.close().await;
            return Err(map_whoami_error(err));
        }

        Ok(Arc::new(session))
    }

    async fn start_interactive_login(
        &self,
        login_id: &LoginId,
        server_name: &str,
        user_id: Option<UserId>,
        redirect_url: &str,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<(Arc<dyn BackendSession>, InteractiveAuthStarted), BackendError> {
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

        let session = Arc::new(BackendSessionMatrix::new(
            client.clone(),
            login_id.clone(),
            self.credential_repository.clone(),
        ));

        if client.oauth().cached_server_metadata().await.is_err() {
            return Ok((session, InteractiveAuthStarted::PasswordRequired));
        }

        let redirect_url =
            Url::parse(redirect_url).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;
        let client_metadata = build_client_metadata(bridge_metadata, redirect_url.clone())?;
        let raw_client_metadata =
            Raw::new(&client_metadata).map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        client
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

        Ok((
            session,
            InteractiveAuthStarted::OAuth {
                auth_url: authorization_data.url.to_string(),
                csrf_token: authorization_data.state.into_secret(),
            },
        ))
    }
}

fn map_whoami_error(error: HttpError) -> BackendError {
    let Some(api_error) = error.client_api_error_kind() else {
        return BackendError::Technical(anyhow!(error));
    };

    match api_error {
        ErrorKind::Forbidden { .. } | ErrorKind::Unauthorized => BackendError::LoggedOut,
        ErrorKind::UnknownToken { soft_logout } => match soft_logout {
            true => BackendError::SoftLoggedOut,
            false => BackendError::LoggedOut,
        },
        _ => BackendError::Technical(anyhow!(error)),
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tachyon_testkit::repositories::CredentialRepositoryInMem;
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    pub(crate) async fn build_test_auth_service() -> (AuthServiceMatrixSdk, MockServer) {
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

        (auth_service, mock_server)
    }

    pub(crate) async fn build_test_client() -> (Client, MockServer) {
        let (auth_service, mock_server) = build_test_auth_service().await;

        let client = auth_service
            .build_client(&ServerName::parse("localhost").unwrap(), None)
            .await
            .unwrap();

        (client, mock_server)
    }

    fn bridge_metadata() -> BridgeMetadata {
        BridgeMetadata {
            name: "Tachyon".to_string(),
            client_uri: "https://localhost/".to_string(),
            image_url: None,
            tos: None,
        }
    }

    #[test]
    fn sanitized_user_id_matches_legacy_store_directory_scheme() {
        let user_id = matrix_sdk::ruma::UserId::parse("@aeon:shlasouf.local").unwrap();
        let sanitized = sanitize_user_id(&user_id);

        assert_eq!(sanitized, "264E4340-A168-537C-890B-946D4EB046E0");
    }

    #[tokio::test]
    async fn start_interactive_login_returns_a_usable_session_on_a_homeserver_without_oauth() {
        let (auth_service, _mock) = build_test_auth_service().await;

        let (session, started) = auth_service
            .start_interactive_login(
                &LoginId::new("l1"),
                "localhost",
                None,
                "https://localhost/callback",
                &bridge_metadata(),
            )
            .await
            .unwrap();

        assert!(matches!(started, InteractiveAuthStarted::PasswordRequired));
        assert!(
            session
                .as_any()
                .downcast_ref::<BackendSessionMatrix>()
                .is_some(),
            "the pending client must come back as a session"
        );
    }
}
