use crate::sqlite_store::{SqliteStore, now_unix};
use async_trait::async_trait;
use rusqlite::{OptionalExtension, params};
use tachyon_core::application::error::StoreError;
use tachyon_core::application::ports::{AccountRepository, StoredLogin};
use tachyon_core::domain::auth::TachyonToken;
use tachyon_core::domain::ids::LoginId;

#[async_trait]
impl AccountRepository for SqliteStore {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError> {
        let token = tachyon_token.as_str().to_owned();
        let login_id: Option<String> = self
            .with_conn(move |conn| {
                conn.query_row(
                    "SELECT login_id FROM tokens WHERE token = ?1",
                    [&token],
                    |row| row.get(0),
                )
                .optional()
            })
            .await?;

        Ok(login_id.map(LoginId::from))
    }

    async fn save_login_for_token(&self, tachyon_token: TachyonToken, login_id: LoginId) -> Result<(), StoreError> {
        let token = tachyon_token.as_str().to_owned();
        let login_id = login_id.to_string();
        let now = now_unix();
        self.with_conn(move |conn| {
            let tx = conn.transaction()?;

            // A backend that persists no credentials can still be linked: the row must
            // exist for the foreign key to hold.
            tx.execute(
                "INSERT OR IGNORE INTO logins (login_id, created_at, updated_at) VALUES (?1, ?2, ?2)",
                params![login_id, now],
            )?;

            tx.execute(
                "INSERT INTO tokens (token, login_id, linked_at) VALUES (?1, ?2, ?3)
                 ON CONFLICT(token) DO UPDATE SET
                     login_id = excluded.login_id,
                     linked_at = excluded.linked_at",
                params![token, login_id, now],
            )?;

            tx.commit()
        })
        .await
    }

    async fn delete_login(&self, login_id: &LoginId) -> Result<(), StoreError> {
        let login_id = login_id.to_string();
        self.with_conn(move |conn| {
            // Tokens go with it: `ON DELETE CASCADE`, and the connection has foreign keys on.
            conn.execute("DELETE FROM logins WHERE login_id = ?1", [&login_id])
                .map(|_| ())
        })
        .await
    }

    async fn logins(&self) -> Result<Vec<StoredLogin>, StoreError> {
        self.with_conn(|conn| {
            let mut statement = conn.prepare(
                "SELECT l.login_id,
                        EXISTS(SELECT 1 FROM tokens t WHERE t.login_id = l.login_id)
                 FROM logins l",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(StoredLogin {
                    login_id: LoginId::from(row.get::<_, String>(0)?),
                    bound: row.get(1)?,
                })
            })?;
            rows.collect()
        })
        .await
    }
}
