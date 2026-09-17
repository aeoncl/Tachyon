use std::path::PathBuf;
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use matrix_sdk::authentication::oauth::registration::{
    ApplicationType, ClientMetadata, Localized, OAuthGrantType,
};
use matrix_sdk::reqwest::Url;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::ruma::api::error::{ErrorKind, UnknownTokenErrorData};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::{Client, HttpError, ServerName};

use tachyon_core::application::error::BackendError;
use tachyon_core::application::ports::{AuthService, BackendSession, CredentialRepository};
use tachyon_core::domain::auth::{BridgeMetadata, InteractiveAuthStarted};
use tachyon_core::domain::ids::{LoginId, UserId};

use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::backend::session::BackendSessionMatrix;
use crate::infrastructure::backend::technical;
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

    fn login_dir(&self, login_id: &LoginId) -> Option<PathBuf> {
        self.config
            .store_root
            .as_ref()
            .map(|root| root.join("logins").join(login_id.to_string()))
    }


    fn has_store(&self, login_id: &LoginId) -> bool {
        match self.login_dir(login_id) {
            None => true,
            Some(login_dir) => login_dir.join("store").exists(),
        }
    }

    async fn build_client(
        &self,
        server_name: &ServerName,
        login_id: &LoginId,
    ) -> Result<(Client, Option<PathBuf>), BackendError> {
        let mut client_builder = Client::builder().handle_refresh_tokens();

        if self.config.disable_ssl {
            client_builder = client_builder.disable_ssl_verification();
        }

        match &self.config.homeserver_url_override {
            None => client_builder = client_builder.server_name(server_name),
            Some(homeserver_url) => client_builder = client_builder.homeserver_url(homeserver_url),
        }

        let login_dir = self.login_dir(login_id);
        if let Some(login_dir) = &login_dir {
            let store_path = login_dir.join("store");
            std::fs::create_dir_all(&store_path).map_err(|e| {
                BackendError::Technical(anyhow!("Could not create store dir: {}", e))
            })?;
            client_builder = client_builder.sqlite_store(store_path, None);
        }

        let client = client_builder
            .build()
            .await
            .map_err(technical)?;
        Ok((client, login_dir))
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

        if !self.has_store(login_id) {
            // Building a client on an empty store would mint fresh identity keys for a device
            // the homeserver already knows under other keys, and nothing can verify that
            // device afterwards. Better to start a new one.
            return Err(BackendError::CannotRestoreLogin(
                "the device's store is missing".to_string(),
            ));
        }
        let (client, login_dir) = self.build_client(user_id.server_name(), login_id).await?;

        let session = BackendSessionMatrix::new(
            client,
            login_id.clone(),
            self.credential_repository.clone(),
            user_id,
            login_dir,
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

        session.sync_until_verified();
        Ok(Arc::new(session))
    }

    async fn start_interactive_login(
        &self,
        login_id: &LoginId,
        server_name: &str,
        user_id: UserId,
        redirect_url: &str,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<(Arc<dyn BackendSession>, InteractiveAuthStarted), BackendError> {
        let user_id: OwnedUserId = user_id
            .map_into()
            .map_err(|e| BackendError::Technical(anyhow!("{:?}", e)))?;

        let server_name =
            ServerName::parse(server_name).map_err(technical)?;

        let (client, login_dir) = self.build_client(&server_name, login_id).await?;

        let session = Arc::new(BackendSessionMatrix::new(
            client.clone(),
            login_id.clone(),
            self.credential_repository.clone(),
            user_id.clone(),
            login_dir,
        ));

        if client.oauth().cached_server_metadata().await.is_err() {
            return Ok((session, InteractiveAuthStarted::PasswordRequired));
        }

        let redirect_url =
            Url::parse(redirect_url).map_err(technical)?;
        let client_metadata = build_client_metadata(bridge_metadata, redirect_url.clone())?;
        let raw_client_metadata =
            Raw::new(&client_metadata).map_err(technical)?;

        client
            .oauth()
            .register_client(&raw_client_metadata)
            .await
            .map_err(technical)?;

        let authorization_data = client
            .oauth()
            .login(redirect_url, None, None, None)
            .user_id_hint(&user_id)
            .build()
            .await
            .map_err(technical)?;

        Ok((
            session,
            InteractiveAuthStarted::OAuth {
                auth_url: authorization_data.url.to_string(),
                csrf_token: authorization_data.state.into_secret(),
            },
        ))
    }

    async fn remove_login(&self, login_id: &LoginId) -> Result<(), BackendError> {
        if let Some(login_dir) = self.login_dir(login_id) {
            remove_login_dir(&login_dir).await?;
        }
        Ok(())
    }

    async fn clear_logins_except(&self, keep: &[LoginId]) -> Result<(), BackendError> {
        let Some(root) = &self.config.store_root else {
            return Ok(());
        };
        let logins = root.join("logins");
        let entries = match std::fs::read_dir(&logins) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => {
                return Err(BackendError::Technical(anyhow!(
                    "Could not list the login stores: {e}"
                )));
            }
        };
        let keep: std::collections::HashSet<String> =
            keep.iter().map(|login_id| login_id.to_string()).collect();

        for entry in entries {
            let entry = entry.map_err(|e| BackendError::Technical(anyhow!("{e}")))?;
            if !keep.contains(&entry.file_name().to_string_lossy().to_string()) {
                remove_login_dir(&entry.path()).await?;
            }
        }
        Ok(())
    }
}

/// The file the SDK's crypto store keeps its keys in; its presence is what makes a store dir
/// a device's store rather than an empty folder.
const CRYPTO_DATABASE: &str = "matrix-sdk-crypto.sqlite3";

/// Nothing to do when the directory is already gone.
pub(crate) async fn remove_login_dir(login_dir: &std::path::Path) -> Result<(), BackendError> {
    match tokio::fs::remove_dir_all(login_dir).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(BackendError::Technical(anyhow!(
            "Could not remove the login store at {}: {e}",
            login_dir.display()
        ))),
    }
}

fn map_whoami_error(error: HttpError) -> BackendError {
    let Some(api_error) = error.client_api_error_kind() else {
        return BackendError::Technical(anyhow!(error));
    };

    match api_error {
        ErrorKind::Forbidden { .. } | ErrorKind::Unauthorized => BackendError::LoggedOut,
        ErrorKind::UnknownToken(UnknownTokenErrorData { soft_logout, .. }) => match soft_logout {
            true => BackendError::SoftLoggedOut,
            false => BackendError::LoggedOut,
        },
        _ => BackendError::Technical(anyhow!(error)),
    }
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

        let (client, _) = auth_service
            .build_client(&ServerName::parse("localhost").unwrap(), &LoginId::new("test"))
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

    /// A store root nobody else writes to, removed when dropped.
    struct StoreRoot(tempfile::TempDir);

    impl StoreRoot {
        fn new() -> Self {
            Self(tempfile::tempdir().unwrap())
        }

        fn login_dir(&self, login_id: &str) -> PathBuf {
            self.0.path().join("logins").join(login_id)
        }
    }

    async fn build_test_auth_service_storing_in(root: &StoreRoot) -> (AuthServiceMatrixSdk, MockServer) {
        let (auth_service, mock_server) = build_test_auth_service().await;
        let auth_service = AuthServiceMatrixSdk::new(
            auth_service.credential_repository.clone(),
            MatrixBackendConfig {
                store_root: Some(root.0.path().to_path_buf()),
                ..auth_service.config
            },
        );
        (auth_service, mock_server)
    }

    async fn stored_login(auth_service: &AuthServiceMatrixSdk, login_id: &str) -> LoginId {
        let login_id = LoginId::new(login_id);
        let blob = SessionRestoreData {
            access_token: "access".to_string(),
            refresh_token: None,
            user_id: matrix_sdk::ruma::UserId::parse("@aeon:shlasouf.local").unwrap().to_owned(),
            device_id: matrix_sdk::ruma::OwnedDeviceId::from("DEVICEID"),
            auth_kind: crate::domain::auth::AuthKind::Matrix,
        }
        .to_blob()
        .unwrap();
        auth_service
            .credential_repository
            .store(&login_id, blob)
            .await
            .unwrap();
        login_id
    }

    #[tokio::test]
    async fn restoring_a_login_whose_store_is_missing_is_refused() {
        let root = StoreRoot::new();
        let (auth_service, _mock) = build_test_auth_service_storing_in(&root).await;
        let login_id = stored_login(&auth_service, "l1").await;

        let refused = auth_service.restore(&login_id).await.err().map(|e| e.to_string());

        assert!(
            refused.as_deref().is_some_and(|e| e.contains("store is missing")),
            "{refused:?}"
        );
        assert!(!root.login_dir("l1").join("store").join(CRYPTO_DATABASE).exists(), "no empty store was minted");
    }

    #[tokio::test]
    async fn every_login_gets_its_own_store_directory() {
        let root = StoreRoot::new();
        let (auth_service, _mock) = build_test_auth_service_storing_in(&root).await;

        for login_id in ["l1", "l2"] {
            auth_service
                .start_interactive_login(
                    &LoginId::new(login_id),
                    "localhost",
                    UserId::new("@aeon:localhost"),
                    "https://localhost/callback",
                    &bridge_metadata(),
                )
                .await
                .unwrap();
        }

        assert!(root.login_dir("l1").join("store").is_dir());
        assert!(root.login_dir("l2").join("store").is_dir());
    }

    #[tokio::test]
    async fn forgetting_a_login_the_backend_cannot_restore_still_removes_its_directory() {
        let root = StoreRoot::new();
        let (auth_service, _mock) = build_test_auth_service_storing_in(&root).await;
        std::fs::create_dir_all(root.login_dir("stale").join("store")).unwrap();

        auth_service.remove_login(&LoginId::new("stale")).await.unwrap();

        assert!(!root.login_dir("stale").exists());
    }

    #[tokio::test]
    async fn sweeping_removes_the_directories_of_logins_not_kept() {
        let root = StoreRoot::new();
        let (auth_service, _mock) = build_test_auth_service_storing_in(&root).await;
        for login_id in ["kept", "stale"] {
            std::fs::create_dir_all(root.login_dir(login_id).join("store")).unwrap();
        }

        auth_service.clear_logins_except(&[LoginId::new("kept")]).await.unwrap();

        assert!(root.login_dir("kept").is_dir());
        assert!(!root.login_dir("stale").exists());
    }

    #[tokio::test]
    async fn sweeping_with_no_login_directory_yet_is_fine() {
        let root = StoreRoot::new();
        let (auth_service, _mock) = build_test_auth_service_storing_in(&root).await;

        auth_service.clear_logins_except(&[]).await.unwrap();
    }

    #[tokio::test]
    async fn discarding_a_login_removes_its_store_directory() {
        let root = StoreRoot::new();
        let (auth_service, _mock) = build_test_auth_service_storing_in(&root).await;
        let (session, _) = auth_service
            .start_interactive_login(
                &LoginId::new("l1"),
                "localhost",
                UserId::new("@aeon:localhost"),
                "https://localhost/callback",
                &bridge_metadata(),
            )
            .await
            .unwrap();
        assert!(root.login_dir("l1").is_dir());

        session.discard().await;
        drop(session);

        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(10);
        while root.login_dir("l1").exists() {
            assert!(tokio::time::Instant::now() < deadline, "the login directory is still there");
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
    }

    #[tokio::test]
    async fn start_interactive_login_returns_a_usable_session_on_a_homeserver_without_oauth() {
        let (auth_service, _mock) = build_test_auth_service().await;

        let (session, started) = auth_service
            .start_interactive_login(
                &LoginId::new("l1"),
                "localhost",
                UserId::new("@aeon:localhost"),
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
