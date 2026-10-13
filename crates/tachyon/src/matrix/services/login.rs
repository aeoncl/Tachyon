use std::path::Path;
use std::sync::Arc;

use log::{debug, error, warn};
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::ruma::api::client::uiaa;
use matrix_sdk::ruma::api::client::uiaa::MatrixUserIdentifier;
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId, ServerName, UserId};
use matrix_sdk::{async_trait, AuthSession, Client, ClientBuilder, SessionTokens};
use tachyon_sqlite_store::{Credentials, CredentialsKind, DeviceId as StoredDeviceId, HomeserverUrl, SqliteCredentialsStore, UserId as StoredUserId};
use tokio::sync::broadcast::Receiver;

use crate::tachyon::config::paths::{create_dir, get_store_path, get_user_data};
use crate::tachyon::error::TachyonError;

#[async_trait]
pub trait MatrixLoginService: Send + Sync {
    async fn restore_saved_session(&self, credentials: &Credentials, homeserver_url: Option<&HomeserverUrl>, disable_ssl: bool) -> Result<Client, TachyonError>;

    async fn login_with_password(&self, matrix_id: &UserId, password: &str, device_id: Option<&StoredDeviceId>, homeserver_url: Option<&HomeserverUrl>, disable_ssl: bool) -> Result<Client, TachyonError>;
}

#[derive(Clone)]
pub struct MatrixLoginServiceImpl {}

impl MatrixLoginServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MatrixLoginService for MatrixLoginServiceImpl {
    async fn restore_saved_session(&self, credentials: &Credentials, homeserver_url: Option<&HomeserverUrl>, disable_ssl: bool) -> Result<Client, TachyonError> {
        let user_id = domain_user_id_to_owned(credentials.user_id())?;
        let store_path = get_store_path(&user_id);

        let client = get_matrix_client_builder(user_id.server_name(), homeserver_url.or(credentials.homeserver_url()), disable_ssl)
            .sqlite_store(store_path, None)
            .build()
            .await?;

        let session = credentials_to_auth_session(credentials)?;
        client.restore_session(session).await?;

        // A dead token must fail the restore loudly; the row is deliberately kept so the re-login reuses its device id.
        if let Err(err) = client.whoami().await {
            if matches!(err.client_api_error_kind(), Some(matrix_sdk::ruma::api::error::ErrorKind::UnknownToken { .. })) {
                warn!("the saved session for {} is dead (unknown token); sign-in required", credentials.user_id());
                return Err(err.into());
            }
            warn!("whoami after restore failed for {} ({}); assuming the session is good", credentials.user_id(), err);
        }
        Ok(client)
    }


    async fn login_with_password(&self, matrix_id: &UserId, password: &str, device_id: Option<&StoredDeviceId>, homeserver_url: Option<&HomeserverUrl>, disable_ssl: bool) -> Result<Client, TachyonError> {
        create_dir(get_user_data(matrix_id).as_path());

        let client = get_matrix_client_builder(matrix_id.server_name(), homeserver_url, disable_ssl)
            .sqlite_store(get_store_path(matrix_id), None)
            .build()
            .await?;

        let mut login_builder = client.matrix_auth()
            .login_username(&matrix_id.to_string(), password)
            .initial_device_display_name("Windows Live Messenger (Tachyon)");

        if let Some(device_id) = device_id {
            login_builder = login_builder.device_id(device_id.as_str())
        }

        let login_result = login_builder.send().await?;

        if let Err(e) = client.encryption().bootstrap_cross_signing_if_needed(None).await {
            if let Some(response) = e.as_uiaa_response() {
                let mut password = uiaa::Password::new(
                    uiaa::UserIdentifier::Matrix(MatrixUserIdentifier::new(login_result.user_id.to_string())),
                    password.to_string(),
                );
                password.session = response.session.clone();

                client
                    .encryption()
                    .bootstrap_cross_signing(Some(uiaa::AuthData::Password(password)))
                    .await
                    .expect("Couldn't bootstrap cross signing")
            } else {
                panic!("Error during cross signing bootstrap {:#?}", e);
            }
        }
        Ok(client)
    }
}

pub fn persist_session(client: &Client, token: &str, store: &SqliteCredentialsStore) {
    let Some(session) = client.session() else {
        return;
    };

    match auth_session_to_credentials(session) {
        Ok(mut credentials) => {
            if let Ok(url) = HomeserverUrl::parse(client.homeserver().as_str()) {
                credentials.set_homeserver_url(url);
            }
            if let Err(e) = store.save(token, &credentials) {
                error!("failed to persist session for {}: {}", token, e);
            }
        }
        Err(e) => error!("failed to map session for {}: {}", token, e),
    }
}

pub fn spawn_session_watcher(client: Client, token: String, store: Arc<SqliteCredentialsStore>, mut shutdown: Receiver<()>) {
    tokio::spawn(async move {
        let mut receiver = client.subscribe_to_session_changes();

        loop {
            tokio::select! {
                _ = shutdown.recv() => break,

                change = receiver.recv() => match change {
                    Ok(change) => {
                        debug!("session change for {}: {:?}", token, change);
                        persist_session(&client, &token, store.as_ref());
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(missed)) => {
                        warn!("session change receiver lagged, missed {} events", missed);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }

        debug!("session watcher for {} stopped", token);
    });
}

pub fn store_hydration_warning(store_path: &Path) -> Option<String> {
    let hydrated = std::fs::read_dir(store_path)
        .map(|entries| {
            entries.filter_map(|entry| entry.ok()).any(|entry| {
                entry.file_name().to_str().is_some_and(|name| name.contains(".sqlite3"))
            })
        })
        .unwrap_or(false);

    (!hydrated).then(|| {
        format!(
            "credentials exist for this token but the crypto store at {} is \
             missing or empty: restoring will reuse the device with fresh keys \
             and one-time-key uploads may be rejected by the server \
             (\"already exists\", 400). Clear the stored session or restore \
             the store folder to fix this properly.",
            store_path.display()
        )
    })
}

pub fn domain_user_id_to_owned(user_id: &StoredUserId) -> Result<OwnedUserId, TachyonError> {
    UserId::parse(format!("@{}:{}", user_id.local_part(), user_id.domain()))
        .map_err(|e| TachyonError::Any(e.into()))
}

fn credentials_to_auth_session(credentials: &Credentials) -> Result<AuthSession, TachyonError> {
    let meta = matrix_sdk::SessionMeta {
        user_id: domain_user_id_to_owned(credentials.user_id())?,
        device_id: OwnedDeviceId::from(credentials.device_id().as_str()),
    };
    let tokens = SessionTokens {
        access_token: credentials.access_token().to_owned(),
        refresh_token: credentials.refresh_token().map(str::to_owned),
    };

    match credentials.kind() {
        CredentialsKind::Password => Ok(MatrixSession { meta, tokens }.into()),
        CredentialsKind::Sso { client_id } => Ok(AuthSession::OAuth(Box::new(matrix_sdk::authentication::oauth::OAuthSession {
            client_id: matrix_sdk::authentication::oauth::ClientId::new(client_id.clone()),
            user: matrix_sdk::authentication::oauth::UserSession { meta, tokens },
        }))),
    }
}

fn auth_session_to_credentials(session: AuthSession) -> Result<Credentials, TachyonError> {
    match session {
        AuthSession::Matrix(session) => Ok(Credentials::password(
            stored_user_id(&session.meta.user_id),
            StoredDeviceId::new(session.meta.device_id.to_string()),
            session.tokens.access_token,
            session.tokens.refresh_token,
        )),
        AuthSession::OAuth(session) => {
            let matrix_sdk::authentication::oauth::OAuthSession { client_id, user } = *session;
            Ok(Credentials::sso(
                stored_user_id(&user.meta.user_id),
                StoredDeviceId::new(user.meta.device_id.to_string()),
                user.tokens.access_token,
                user.tokens.refresh_token,
                client_id.as_str().to_owned(),
            ))
        }
        _ => Err(TachyonError::Any(anyhow::anyhow!("unsupported auth session kind"))),
    }
}

fn stored_user_id(user_id: &UserId) -> StoredUserId {
    StoredUserId::new(user_id.localpart().to_owned(), user_id.server_name().to_string())
}

pub(crate) fn get_matrix_client_builder(server_name: &ServerName, homeserver_url: Option<&HomeserverUrl>, disable_ssl: bool) -> ClientBuilder {
    let mut client_builder = Client::builder();

    client_builder = client_builder.handle_refresh_tokens();

    if disable_ssl {
        client_builder = client_builder.disable_ssl_verification();
    }

    match homeserver_url {
        None => {
            client_builder = client_builder.server_name(server_name)
        }
        Some(homeserver_url) => {
            client_builder = client_builder.homeserver_url(homeserver_url.as_str())
        }
    }

    client_builder
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tachyon::identifiers::bridge_link::{BridgeLinkToken, BridgeType, ClientVersion};
    use matrix_sdk::ruma::api::error::ErrorKind;
    use std::path::PathBuf;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // TACHYON_HOME is process-global; the restore tests must not interleave
    static TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn stored_credentials(access_token: &str) -> Credentials {
        Credentials::password(
            StoredUserId::new("alice".to_string(), "example.org".to_string()),
            StoredDeviceId::new("DEVOLD".to_string()),
            access_token.to_string(),
            None,
        )
    }

    fn temp_home(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("tachyon-login-test-{}", std::process::id())).join(test);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create home");
        std::env::set_var("TACHYON_HOME", &dir);
        dir
    }

    // The SDK's send pipeline refreshes its supported-versions cache first; without a 200 there every request dies as a cached 404.
    fn versions_mock() -> Mock {
        Mock::given(method("GET"))
            .and(path("/_matrix/client/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "versions": ["v1.11"] })))
    }

    fn whoami_mock(status: u16, body: serde_json::Value) -> Mock {
        Mock::given(method("GET"))
            .and(path("/_matrix/client/v3/account/whoami"))
            .respond_with(ResponseTemplate::new(status).set_body_json(body))
    }

    async fn restore(credentials: &Credentials, server_uri: &str) -> Result<Client, TachyonError> {
        let homeserver = HomeserverUrl::parse(server_uri).unwrap();
        MatrixLoginServiceImpl::new()
            .restore_saved_session(credentials, Some(&homeserver), true)
            .await
    }

    #[tokio::test]
    async fn a_rejected_token_fails_the_restore_and_keeps_the_row() {
        let _lock = TEST_LOCK.lock().await;
        let home = temp_home("dead-token");
        let store = SqliteCredentialsStore::open(&home.join("credentials.db")).unwrap();
        let token = BridgeLinkToken::from_parts(BridgeType::Msn, "alice@example.org", ClientVersion(14));
        store.save(token.as_str(), &stored_credentials("the-saved-access-token")).unwrap();

        let server = MockServer::start().await;
        versions_mock().mount(&server).await;
        whoami_mock(
            401,
            serde_json::json!({ "errcode": "M_UNKNOWN_TOKEN", "error": "invalid token", "soft_logout": false }),
        )
        .mount(&server)
        .await;

        let outcome = restore(&stored_credentials("the-saved-access-token"), server.uri().as_str()).await;

        let err = outcome.expect_err("a rejected token must fail the restore");
        let api_kind = match &err {
            TachyonError::MatrixError(e) => e.client_api_error_kind().cloned(),
            TachyonError::HttpError(e) => e.client_api_error_kind().cloned(),
            other => panic!("expected a matrix error, got {:?}", other),
        };
        assert!(
            matches!(api_kind, Some(ErrorKind::UnknownToken { .. })),
            "expected UnknownToken, got {:?}",
            api_kind
        );
        assert!(
            store.load(token.as_str()).unwrap().is_some(),
            "the dead-token path must keep the credentials row"
        );
    }

    #[tokio::test]
    async fn a_live_token_restores() {
        let _lock = TEST_LOCK.lock().await;
        temp_home("live-token");
        let server = MockServer::start().await;
        versions_mock().mount(&server).await;
        whoami_mock(200, serde_json::json!({ "user_id": "@alice:example.org" })).mount(&server).await;

        let client = restore(&stored_credentials("live-token"), server.uri().as_str())
            .await
            .expect("credentials existed and the token is alive");
        assert!(client.session().is_some(), "the restored session is installed");
    }

    #[tokio::test]
    async fn an_error_that_is_not_unknown_token_keeps_the_session_installed() {
        let _lock = TEST_LOCK.lock().await;
        temp_home("other-error");
        let server = MockServer::start().await;
        versions_mock().mount(&server).await;
        whoami_mock(400, serde_json::json!({ "errcode": "M_UNRECOGNIZED", "error": "oops" })).mount(&server).await;

        let client = restore(&stored_credentials("some-token"), server.uri().as_str())
            .await
            .expect("a non-token whoami failure must not fail the restore");
        assert!(client.session().is_some(), "the session stays installed");
    }

    #[tokio::test]
    async fn dropping_the_client_releases_the_store_folder() {
        let _lock = TEST_LOCK.lock().await;
        let _home = temp_home("store-release");
        let server = MockServer::start().await;
        versions_mock().mount(&server).await;
        whoami_mock(200, serde_json::json!({ "user_id": "@alice:example.org" })).mount(&server).await;

        let client = restore(&stored_credentials("release-token"), server.uri().as_str()).await.expect("builds");
        let user_data = get_user_data(&UserId::parse("@alice:example.org").unwrap());
        let marker = user_data.join("store").join("locked-if-open.marker");
        std::fs::write(&marker, b"probe").expect("write probe into store dir");

        drop(client);
        // Background tasks observe cancellation asynchronously; give them a moment.
        tokio::time::sleep(std::time::Duration::from_millis(750)).await;

        // On Windows renaming a folder with an open handle fails; the rename is the assertion that every store handle is released.
        let renamed = user_data.with_extension("renamed");
        std::fs::rename(&user_data, &renamed).expect("store folder must be renameable after the client dropped (no open handles)");
        let _ = std::fs::remove_dir_all(&renamed);
    }

    #[tokio::test]
    async fn persist_session_round_trips_through_the_store() {
        let _lock = TEST_LOCK.lock().await;
        let home = temp_home("persist");
        let store = SqliteCredentialsStore::open(&home.join("credentials.db")).unwrap();
        let server = MockServer::start().await;
        versions_mock().mount(&server).await;
        whoami_mock(200, serde_json::json!({ "user_id": "@alice:example.org" })).mount(&server).await;

        let client = restore(&stored_credentials("persist-token"), server.uri().as_str()).await.unwrap();
        let token = BridgeLinkToken::from_parts(BridgeType::Msn, "alice@example.org", ClientVersion(14));
        persist_session(&client, token.as_str(), &store);

        let loaded = store.load(token.as_str()).unwrap().expect("row saved");
        assert_eq!(loaded.access_token(), "persist-token");
        assert_eq!(loaded.user_id().to_string(), "alice@example.org");
        assert_eq!(loaded.device_id().as_str(), "DEVOLD");
        assert_eq!(loaded.homeserver_url().map(HomeserverUrl::as_str), Some(server.uri().as_str()));
    }

    #[test]
    fn missing_or_empty_store_warns_and_a_hydrated_one_does_not() {
        let base = std::env::temp_dir().join(format!("tachyon-hydration-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);

        assert!(store_hydration_warning(&base.join("absent")).is_some());

        std::fs::create_dir_all(base.join("empty")).unwrap();
        assert!(store_hydration_warning(&base.join("empty")).is_some());

        std::fs::create_dir_all(&base).unwrap();
        std::fs::write(base.join("matrix-sdk-state.sqlite3"), b"x").unwrap();
        assert!(store_hydration_warning(&base).is_none());
        let _ = std::fs::remove_dir_all(&base);
    }
}
