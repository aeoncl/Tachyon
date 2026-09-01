use crate::schema;
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tachyon_core::application::error::StoreError;

/// One SQLite database holding everything Tachyon persists itself (matrix-sdk keeps its
/// own store per account). Cheap to clone; all clones share the single connection.
#[derive(Clone)]
pub struct SqliteStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StoreError::Technical(anyhow::anyhow!("could not create the store dir: {e}"))
            })?;
        }
        Self::init(Connection::open(path).map_err(technical)?)
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        Self::init(Connection::open_in_memory().map_err(technical)?)
    }

    fn init(mut conn: Connection) -> Result<Self, StoreError> {
        conn.pragma_update(None, "journal_mode", "WAL").map_err(technical)?;
        // Per-connection pragma: fine while the store shares one connection; must move to a
        // per-connection hook if this ever grows a pool.
        conn.pragma_update(None, "foreign_keys", "ON").map_err(technical)?;
        conn.busy_timeout(std::time::Duration::from_millis(5000))
            .map_err(technical)?;
        schema::migrate(&mut conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Runs `f` with the connection on the blocking pool; the mutex is only ever locked
    /// inside that closure.
    pub(crate) async fn with_conn<T, F>(&self, f: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let mut guard = conn
                .lock()
                .map_err(|_| StoreError::Technical(anyhow::anyhow!("store mutex poisoned")))?;
            f(&mut guard).map_err(technical)
        })
        .await
        .map_err(|e| StoreError::Technical(anyhow::anyhow!("store task failed: {e}")))?
    }

    #[cfg(test)]
    pub(crate) fn user_version(&self) -> i64 {
        self.conn
            .lock()
            .unwrap()
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap()
    }
}

pub(crate) fn technical(e: rusqlite::Error) -> StoreError {
    StoreError::Technical(anyhow::Error::new(e))
}

pub(crate) fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
