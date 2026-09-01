use crate::sqlite_store::{SqliteStore, now_unix};
use async_trait::async_trait;
use rusqlite::OptionalExtension;
use tachyon_core::application::error::StoreError;
use tachyon_core::application::ports::CredentialRepository;
use tachyon_core::domain::auth::CredentialBlob;
use tachyon_core::domain::ids::LoginId;

/// How the bytes are encoded at rest. Owned by the store; it says nothing about what the
/// backend adapter serialized into them.
const FORMAT_PLAINTEXT: i64 = 0;

#[async_trait]
impl CredentialRepository for SqliteStore {
    async fn credentials(&self, login_id: &LoginId) -> Result<Option<CredentialBlob>, StoreError> {
        let login_id = login_id.to_string();
        let row: Option<(Option<Vec<u8>>, i64)> = self
            .with_conn(move |conn| {
                conn.query_row(
                    "SELECT credentials, credentials_format FROM logins WHERE login_id = ?1",
                    [&login_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()
            })
            .await?;

        // `link` creates credential-less rows, so a NULL blob is as absent as a missing row.
        match row {
            None | Some((None, _)) => Ok(None),
            Some((Some(bytes), FORMAT_PLAINTEXT)) => Ok(Some(CredentialBlob::new(bytes))),
            Some((Some(_), format)) => Err(StoreError::Corrupted(format!(
                "credentials format {format} is not readable by this build"
            ))),
        }
    }

    async fn store(&self, login_id: &LoginId, blob: CredentialBlob) -> Result<(), StoreError> {
        let login_id = login_id.to_string();
        let now = now_unix();
        self.with_conn(move |conn| {
            conn.execute(
                "INSERT INTO logins (login_id, credentials, credentials_format, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?4)
                 ON CONFLICT(login_id) DO UPDATE SET
                     credentials = excluded.credentials,
                     credentials_format = excluded.credentials_format,
                     updated_at = excluded.updated_at",
                rusqlite::params![login_id, blob.as_bytes(), FORMAT_PLAINTEXT, now],
            )
            .map(|_| ())
        })
        .await
    }
}
