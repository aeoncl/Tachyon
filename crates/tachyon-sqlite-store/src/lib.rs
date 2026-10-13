use std::fmt::{Display, Formatter};
use std::path::Path;
use std::sync::{Mutex, MutexGuard};

use anyhow::{Context, Result};
use rusqlite::{Connection, OptionalExtension, params};
use url::Url;

const KIND_PASSWORD: &str = "password";
const KIND_SSO: &str = "sso";

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct UserId {
    local_part: String,
    domain: String,
}

impl UserId {
    pub fn new(local_part: String, domain: String) -> Self {
        Self { local_part, domain }
    }

    pub fn parse(value: &str) -> Option<Self> {
        let (local_part, domain) = value.split_once('@')?;
        if local_part.is_empty() || domain.is_empty() {
            return None;
        }
        Some(Self::new(local_part.to_string(), domain.to_string()))
    }

    pub fn local_part(&self) -> &str {
        &self.local_part
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }
}

impl Display for UserId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.local_part, self.domain)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HomeserverUrl(String);

impl HomeserverUrl {
    pub fn parse(input: &str) -> Result<Self> {
        let trimmed = input.trim();
        let url = Url::parse(trimmed).map_err(|e| anyhow::anyhow!("{trimmed:?}: {e}"))?;
        if url.scheme() != "http" && url.scheme() != "https" {
            anyhow::bail!("{trimmed:?} must use http or https");
        }
        if url.host_str().map(str::is_empty).unwrap_or(true) {
            anyhow::bail!("{trimmed:?} has no host");
        }
        if !url.query().unwrap_or_default().is_empty() || !url.fragment().unwrap_or_default().is_empty() {
            anyhow::bail!("{trimmed:?} must not carry a query or fragment");
        }
        let mut canonical = url;
        canonical.set_query(None);
        canonical.set_fragment(None);
        let mut canonical = canonical.to_string();
        while canonical.ends_with('/') {
            canonical.pop();
        }
        Ok(Self(canonical))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for HomeserverUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug)]
pub struct Credentials {
    user_id: UserId,
    device_id: DeviceId,
    access_token: String,
    refresh_token: Option<String>,
    kind: CredentialsKind,
    homeserver_url: Option<HomeserverUrl>,
}

impl Credentials {
    pub fn sso(user_id: UserId, device_id: DeviceId, access_token: String, refresh_token: Option<String>, client_id: String) -> Self {
        Self { user_id, device_id, access_token, refresh_token, kind: CredentialsKind::Sso { client_id }, homeserver_url: None }
    }

    pub fn password(user_id: UserId, device_id: DeviceId, access_token: String, refresh_token: Option<String>) -> Self {
        Self { user_id, device_id, access_token, refresh_token, kind: CredentialsKind::Password, homeserver_url: None }
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn device_id(&self) -> &DeviceId {
        &self.device_id
    }

    pub fn access_token(&self) -> &str {
        &self.access_token
    }

    pub fn refresh_token(&self) -> Option<&str> {
        self.refresh_token.as_deref()
    }

    pub fn kind(&self) -> &CredentialsKind {
        &self.kind
    }

    pub fn homeserver_url(&self) -> Option<&HomeserverUrl> {
        self.homeserver_url.as_ref()
    }

    pub fn set_homeserver_url(&mut self, url: HomeserverUrl) {
        self.homeserver_url = Some(url);
    }
}

#[derive(Clone, Debug)]
pub enum CredentialsKind {
    Sso { client_id: String },
    Password,
}

pub struct SqliteCredentialsStore {
    conn: Mutex<Connection>,
}

impl SqliteCredentialsStore {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).with_context(|| format!("failed to create database directory {}", parent.display()))?;
        }
        let conn = Connection::open(path).with_context(|| format!("failed to open sqlite database {}", path.display()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS credentials (
                 token           TEXT PRIMARY KEY,
                 user_id_local   TEXT NOT NULL,
                 user_id_domain  TEXT NOT NULL,
                 device_id       TEXT NOT NULL,
                 access_token    TEXT NOT NULL,
                 refresh_token   TEXT NULL,
                 kind            TEXT NOT NULL,
                 oauth_client_id TEXT NULL,
                 homeserver_url  TEXT NULL
             );",
        )
        .context("failed to create credentials table")?;
        // CREATE TABLE IF NOT EXISTS is a no-op on pre-existing databases, so the column is added here
        let has_column: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('credentials')
                 WHERE name = 'homeserver_url'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .context("failed to inspect credentials table")?;
        if !has_column {
            conn.execute_batch("ALTER TABLE credentials ADD COLUMN homeserver_url TEXT NULL;")
                .context("failed to add homeserver_url column")?;
        }
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn conn(&self) -> Result<MutexGuard<'_, Connection>> {
        self.conn.lock().map_err(|_| anyhow::anyhow!("sqlite connection mutex poisoned"))
    }

    pub fn load(&self, token: &str) -> Result<Option<Credentials>> {
        let conn = self.conn()?;
        let row = conn
            .query_row(
                "SELECT user_id_local, user_id_domain, device_id, access_token,
                        refresh_token, kind, oauth_client_id, homeserver_url
                 FROM credentials WHERE token = ?1",
                params![token],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, String>(5)?,
                        row.get::<_, Option<String>>(6)?,
                        row.get::<_, Option<String>>(7)?,
                    ))
                },
            )
            .optional()
            .context("failed to load credentials")?;
        row.map(credentials_from_row).transpose()
    }

    pub fn save(&self, token: &str, credentials: &Credentials) -> Result<()> {
        let conn = self.conn()?;
        let (kind, oauth_client_id) = match credentials.kind() {
            CredentialsKind::Password => (KIND_PASSWORD, None),
            CredentialsKind::Sso { client_id } => (KIND_SSO, Some(client_id.clone())),
        };
        conn.execute(
            "INSERT OR REPLACE INTO credentials
                 (token, user_id_local, user_id_domain, device_id, access_token,
                  refresh_token, kind, oauth_client_id, homeserver_url)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                token,
                credentials.user_id().local_part(),
                credentials.user_id().domain(),
                credentials.device_id().as_str(),
                credentials.access_token(),
                credentials.refresh_token(),
                kind,
                oauth_client_id,
                credentials.homeserver_url().map(|url| url.as_str().to_string()),
            ],
        )
        .context("failed to save credentials")?;
        Ok(())
    }

    pub fn clear(&self, token: &str) -> Result<()> {
        let conn = self.conn()?;
        conn.execute("DELETE FROM credentials WHERE token = ?1", params![token])
            .context("failed to clear credentials")?;
        Ok(())
    }
}

type CredentialsRow = (String, String, String, String, Option<String>, String, Option<String>, Option<String>);

fn credentials_from_row(
    (user_id_local, user_id_domain, device_id, access_token, refresh_token, kind, oauth_client_id, homeserver_url): CredentialsRow,
) -> Result<Credentials> {
    let user_id = UserId::new(user_id_local, user_id_domain);
    let device_id = DeviceId::new(device_id);
    let mut credentials = match kind.as_str() {
        KIND_PASSWORD => Credentials::password(user_id, device_id, access_token, refresh_token),
        KIND_SSO => {
            let client_id = oauth_client_id.ok_or_else(|| {
                anyhow::anyhow!("corrupt credentials row: sso kind without oauth client id")
            })?;
            Credentials::sso(user_id, device_id, access_token, refresh_token, client_id)
        }
        other => anyhow::bail!("corrupt credentials row: unknown kind '{}'", other),
    };
    if let Some(url) = homeserver_url {
        if let Ok(parsed) = HomeserverUrl::parse(&url) {
            credentials.set_homeserver_url(parsed);
        }
    }
    Ok(credentials)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDir(std::path::PathBuf);

    impl TestDir {
        fn new(test_name: &str) -> Self {
            Self(std::env::temp_dir().join(format!("tachyon-test-{}", std::process::id())).join(test_name))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn token(value: &str) -> String {
        value.to_string()
    }

    fn password_credentials(refresh_token: Option<String>) -> Credentials {
        Credentials::password(
            UserId::new("alice".to_string(), "example.org".to_string()),
            DeviceId::new("DEVICE123".to_string()),
            "access-token-password".to_string(),
            refresh_token,
        )
    }

    fn sso_credentials(refresh_token: Option<String>) -> Credentials {
        Credentials::sso(
            UserId::new("bob".to_string(), "example.org".to_string()),
            DeviceId::new("DEVICE456".to_string()),
            "access-token-sso".to_string(),
            refresh_token,
            "oauth-client-abc".to_string(),
        )
    }

    fn open_store(dir: &TestDir) -> SqliteCredentialsStore {
        SqliteCredentialsStore::open(&dir.path().join("nested").join("credentials.db")).expect("store opens")
    }

    fn assert_round_trip(store: &SqliteCredentialsStore, token: &str, original: &Credentials) {
        store.save(token, original).expect("save");
        let loaded = store.load(token).expect("load").expect("some credentials");
        assert_eq!(loaded.user_id(), original.user_id());
        assert_eq!(loaded.device_id(), original.device_id());
        assert_eq!(loaded.access_token(), original.access_token());
        assert_eq!(loaded.refresh_token(), original.refresh_token());
        assert_eq!(loaded.homeserver_url().map(HomeserverUrl::as_str), original.homeserver_url().map(HomeserverUrl::as_str));
        match (loaded.kind(), original.kind()) {
            (CredentialsKind::Password, CredentialsKind::Password) => {}
            (CredentialsKind::Sso { client_id: loaded_id }, CredentialsKind::Sso { client_id: original_id }) => {
                assert_eq!(loaded_id, original_id)
            }
            _ => panic!("credentials kind mismatch after round trip"),
        }
    }

    #[test]
    fn password_credentials_round_trip_with_and_without_refresh_token() {
        let dir = TestDir::new("password-round-trip");
        let store = open_store(&dir);
        assert_round_trip(&store, &token("token-password"), &password_credentials(Some("refresh-1".to_string())));
        assert_round_trip(&store, &token("token-password"), &password_credentials(None));
    }

    #[test]
    fn sso_credentials_round_trip_with_and_without_refresh_token() {
        let dir = TestDir::new("sso-round-trip");
        let store = open_store(&dir);
        assert_round_trip(&store, &token("token-sso"), &sso_credentials(Some("refresh-2".to_string())));
        assert_round_trip(&store, &token("token-sso"), &sso_credentials(None));
    }

    #[test]
    fn load_unknown_token_returns_none() {
        let dir = TestDir::new("load-unknown");
        let store = open_store(&dir);
        assert!(store.load(&token("never-saved")).expect("load").is_none());
    }

    #[test]
    fn save_twice_overwrites_row_instead_of_appending() {
        let dir = TestDir::new("save-overwrites");
        let store = open_store(&dir);
        let token = token("token-overwrite");
        store.save(&token, &password_credentials(Some("refresh-old".to_string()))).expect("first save");
        store.save(&token, &sso_credentials(None)).expect("second save");

        let loaded = store.load(&token).expect("load").expect("some credentials");
        assert_eq!(loaded.access_token(), "access-token-sso");
        assert_eq!(loaded.user_id().local_part(), "bob");
        assert!(matches!(loaded.kind(), CredentialsKind::Sso { .. }));

        let count: i64 = store
            .conn
            .lock()
            .expect("lock")
            .query_row("SELECT COUNT(*) FROM credentials", [], |row| row.get(0))
            .expect("count rows");
        assert_eq!(count, 1);
    }

    #[test]
    fn clear_removes_saved_credentials_and_unknown_token_is_a_noop() {
        let dir = TestDir::new("clear-removes");
        let store = open_store(&dir);
        store.save(&token("token-clear"), &password_credentials(None)).expect("save");
        assert!(store.load(&token("token-clear")).expect("load").is_some());

        store.clear(&token("token-clear")).expect("clear");
        assert!(store.load(&token("token-clear")).expect("load after clear").is_none());

        store.clear(&token("unknown")).expect("clear unknown");
    }

    #[test]
    fn homeserver_url_round_trips_and_null_stays_none() {
        let dir = TestDir::new("homeserver-url-round-trip");
        let store = open_store(&dir);
        let token = token("token-homeserver");

        let mut credentials = password_credentials(None);
        credentials.set_homeserver_url(HomeserverUrl::parse("https://matrix.example.org/").expect("parses"));
        store.save(&token, &credentials).expect("save");
        let loaded = store.load(&token).expect("load").expect("some credentials");
        assert_eq!(loaded.homeserver_url().map(HomeserverUrl::as_str), Some("https://matrix.example.org"));

        store.save(&token, &password_credentials(None)).expect("resave");
        let loaded = store.load(&token).expect("load").expect("some credentials");
        assert!(loaded.homeserver_url().is_none());
    }

    #[test]
    fn pre_homeserver_database_is_migrated_on_open() {
        let dir = TestDir::new("migration-legacy-db");
        std::fs::create_dir_all(dir.path()).expect("test dir created");
        let db_path = dir.path().join("credentials.db");
        {
            let conn = rusqlite::Connection::open(&db_path).expect("legacy db opens");
            conn.execute_batch(
                "CREATE TABLE credentials (
                     token           TEXT PRIMARY KEY,
                     user_id_local   TEXT NOT NULL,
                     user_id_domain  TEXT NOT NULL,
                     device_id       TEXT NOT NULL,
                     access_token    TEXT NOT NULL,
                     refresh_token   TEXT NULL,
                     kind            TEXT NOT NULL,
                     oauth_client_id TEXT NULL
                 );
                 INSERT INTO credentials VALUES ('legacy-token', 'alice', 'example.org',
                     'DEVICEOLD', 'access-old', NULL, 'password', NULL);",
            )
            .expect("legacy schema created");
        }

        let store = SqliteCredentialsStore::open(&db_path).expect("open migrates");
        let loaded = store.load(&token("legacy-token")).expect("load").expect("legacy row survives");
        assert_eq!(loaded.access_token(), "access-old");
        assert!(loaded.homeserver_url().is_none());
    }

    #[test]
    fn homeserver_url_parse_canonicalizes_and_rejects_garbage() {
        assert_eq!(
            HomeserverUrl::parse("HTTPS://Matrix.Example.org:8448/").unwrap().as_str(),
            "https://matrix.example.org:8448"
        );
        assert_eq!(
            HomeserverUrl::parse("https://example.org/base-path/").unwrap().as_str(),
            "https://example.org/base-path"
        );
        assert!(HomeserverUrl::parse("example.org").is_err());
        assert!(HomeserverUrl::parse("ftp://example.org").is_err());
        assert!(HomeserverUrl::parse("https://example.org/?x=1").is_err());
        assert!(HomeserverUrl::parse("").is_err());
    }

    #[test]
    fn user_id_parse_splits_on_at() {
        let user_id = UserId::parse("alice@example.org").expect("parses");
        assert_eq!((user_id.local_part(), user_id.domain()), ("alice", "example.org"));
        assert_eq!(user_id.to_string(), "alice@example.org");
        assert!(UserId::parse("alice@").is_none());
        assert!(UserId::parse("@example.org").is_none());
        assert!(UserId::parse("no-at-sign").is_none());
    }
}
