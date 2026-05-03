use anyhow::anyhow;
use log::debug;

use matrix_sdk::{async_trait, AuthSession, Client, ClientBuilder, ServerName, SessionTokens};
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::ruma::{device_id, OwnedUserId};
use matrix_sdk::ruma::api::client::uiaa;
use matrix_sdk::ruma::api::client::uiaa::AuthData;
use matrix_sdk_ui::sync_service::SyncService;
use matrix_sdk::ruma::UserId;
use msnp::shared::models::ticket_token::TicketToken;

use crate::tachyon::error::{MatrixConversionError, TachyonError};
use crate::tachyon::global::paths::get_store_path;
use crate::tachyon::identifiers::tachyon_device_id::TachyonDeviceId;

pub(crate) type AccessToken = String;

#[async_trait]
pub trait MatrixLoginService: Send + Sync {
    async fn login_with_token(&self, user_id: &UserId, token: &str, disable_ssl: bool) -> Result<Client, TachyonError>;

    async fn login_with_password(&self, matrix_id: &UserId, password: &str, disable_ssl: bool) -> Result<(AccessToken, Client), TachyonError>;
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
    async fn login_with_token(&self, user_id: &UserId, token: &str, disable_ssl: bool) -> Result<Client, TachyonError> {
        let device_id_str = get_device_id()?.to_string();
        let device_id = device_id!(device_id_str.as_str()).to_owned();

        let _test: SyncService;

        let store_path = get_store_path(user_id).ok_or(anyhow!("Couldn't get store path"))?;
        debug!("storepath: {:?}", &store_path);

        let client = get_matrix_client_builder(user_id.server_name(), None, disable_ssl)
            .sqlite_store(store_path, None)
            .build()
            .await?;

        client.restore_session(AuthSession::Matrix(MatrixSession {
            meta: matrix_sdk::SessionMeta { user_id: user_id.to_owned(), device_id },
            tokens: SessionTokens { access_token: token.to_string(), refresh_token: None },
        })).await?;

        client.whoami().await?;
        Ok(client)
    }


    async fn login_with_password(&self, matrix_id: &UserId, password: &str, disable_ssl: bool) -> Result<(AccessToken, Client), TachyonError> {
        let client = get_matrix_client_builder(matrix_id.server_name(), None, disable_ssl).build().await?;

        let device_id = get_device_id()?;
        let device_id_as_str = device_id.to_string();

        let result = client.matrix_auth()
            .login_username(&matrix_id, password)
            .device_id(&device_id_as_str)
            .initial_device_display_name(get_device_display_name(&device_id).as_str())
            .send()
            .await?;

        if let Err(e) = client.encryption().bootstrap_cross_signing_if_needed(None).await {
            if let Some(response) = e.as_uiaa_response() {
                let mut password = uiaa::Password::new(
                    uiaa::UserIdentifier::UserIdOrLocalpart(matrix_id.to_string()),
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

        Ok((result.access_token, client))
    }
}

fn get_device_id() -> Result<TachyonDeviceId, MatrixConversionError> {
    TachyonDeviceId::from_hostname()
}

fn get_device_display_name(device_id: &TachyonDeviceId) -> String {
    format!("Tachyon-{}", &device_id)
}

fn get_matrix_client_builder(server_name: &ServerName, homeserver_url: Option<String>, disable_ssl: bool) -> ClientBuilder {
    let mut client_builder = Client::builder();

    if disable_ssl {
        client_builder = client_builder.disable_ssl_verification();
    }

    match homeserver_url {
        None => {
            client_builder = client_builder.server_name(server_name)
        }
        Some(homeserver_url) => {
            client_builder = client_builder.homeserver_url(&homeserver_url)
        }
    }

    client_builder
}