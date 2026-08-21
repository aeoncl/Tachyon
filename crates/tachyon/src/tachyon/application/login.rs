use anyhow::anyhow;
use matrix_sdk::async_trait;
use ruma::api::auth_scheme::AccessToken;
use ruma::OwnedUserId;
use crate::matrix::application::login::MatrixLoginService;

pub type TachyonToken = String;

#[async_trait]
pub trait TachyonLoginService {

    async fn restore_session(&self, access_token: TachyonToken) -> Result<(), anyhow::Error>;

}

pub trait TachyonCredentialsRepository {
    fn get_credentials(&self, tachyon_token: &TachyonToken) -> Result<Option<MatrixCredentials>, anyhow::Error>;
    fn update_credentials(&self, tachyon_token: TachyonToken, matrix_credentials: MatrixCredentials);

    fn create_credentials(&self, tachyon_token: TachyonToken, matrix_credentials: MatrixCredentials) -> Result<(), anyhow::Error>;

    fn remove_credentials(&self, tachyon_token: &TachyonToken) -> Result<(), anyhow::Error>;
}

#[derive(Clone)]
pub struct MatrixCredentials {
    pub user_id: OwnedUserId,
    pub access_token: String,
    pub refresh_token: Option<String>
}


pub(super) struct TachyonLoginServiceImpl {
    credentials_repository: Box<dyn TachyonCredentialsRepository>,
    matrix_login_service: Box<dyn MatrixLoginService>,
}

impl TachyonLoginServiceImpl {
    pub fn new(credentials_repository: Box<dyn TachyonCredentialsRepository>, matrix_login_service: Box<dyn MatrixLoginService>) -> Self {
        Self {
            credentials_repository,
            matrix_login_service,
        }
    }
}

#[async_trait]
impl TachyonLoginService for TachyonLoginServiceImpl {
    async fn restore_session(&self, access_token: TachyonToken) -> Result<(), anyhow::Error> {
        let Some(credentials) = self.credentials_repository.get_credentials(&access_token)? else {
            return Err(anyhow!("Could not find credentials to restore session with"));
        };

        let matrix_client = self.matrix_login_service.login_with_token(&credentials.user_id, &credentials.access_token, credentials.refresh_token, false).await?;

        //TODO Store matrix client

        Ok(())
    }
}