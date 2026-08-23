use async_trait::async_trait;

use crate::{
    domain::{auth::TachyonToken, ids::LoginId},
    port::error::StoreError,
};

#[async_trait]
pub trait AccountStore {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError>;
}
