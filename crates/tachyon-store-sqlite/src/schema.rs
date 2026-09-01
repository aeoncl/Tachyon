use crate::sqlite_store::technical;
use rusqlite::Connection;
use tachyon_core::application::error::StoreError;

/// One entry per schema version; entry `n` migrates `user_version` `n` to `n + 1`.
/// Never edit a shipped entry; append.
pub(crate) const MIGRATIONS: &[&str] = &["
    CREATE TABLE logins (
        login_id            TEXT PRIMARY KEY,
        credentials         BLOB,                       -- NULL until the backend stored one
        credentials_format  INTEGER NOT NULL DEFAULT 0, -- 0 = plaintext; reserved for encryption at rest
        created_at          INTEGER NOT NULL,
        updated_at          INTEGER NOT NULL
    );

    CREATE TABLE tokens (
        token      TEXT PRIMARY KEY,
        login_id   TEXT NOT NULL REFERENCES logins(login_id) ON DELETE CASCADE,
        linked_at  INTEGER NOT NULL
    );
"];

pub(crate) fn migrate(conn: &mut Connection) -> Result<(), StoreError> {
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(technical)?;

    for (i, migration) in MIGRATIONS.iter().enumerate().skip(version as usize) {
        let tx = conn.transaction().map_err(technical)?;
        tx.execute_batch(migration).map_err(technical)?;
        tx.pragma_update(None, "user_version", (i + 1) as i64)
            .map_err(technical)?;
        tx.commit().map_err(technical)?;
    }

    Ok(())
}
