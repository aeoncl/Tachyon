use anyhow::{anyhow, Error};
use log::{debug, error};
use std::fs;

use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::ruma::api::client::uiaa;
use matrix_sdk::ruma::api::client::uiaa::AuthData;
use matrix_sdk::ruma::UserId;
use matrix_sdk::ruma::{device_id, DeviceId, OwnedDeviceId, OwnedUserId};
use matrix_sdk::{async_trait, AuthSession, Client, ClientBuilder, ServerName, SessionTokens};
use matrix_sdk_ui::sync_service::SyncService;
use tokio::fs::create_dir_all;
use msnp::shared::models::ticket_token::TicketToken;

use crate::tachyon::config::paths::{create_dir, create_dirs, get_store_path, get_user_data};
use crate::tachyon::error::{MatrixConversionError, TachyonError};
use crate::tachyon::identifiers::tachyon_device_id::TachyonDeviceId;

pub(crate) type AccessToken = String;

#[async_trait]
pub trait MatrixLoginService: Send + Sync {
    async fn login_with_token(&self, user_id: &UserId, token: &str, refresh_token: Option<String>, disable_ssl: bool) -> Result<Client, TachyonError>;

    async fn login_with_password(&self, matrix_id: &UserId, password: &str, disable_ssl: bool) -> Result<(AccessToken, Client), TachyonError>;
}

#[derive(Clone)]
pub(super) struct MatrixLoginServiceImpl {}

impl MatrixLoginServiceImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl MatrixLoginService for MatrixLoginServiceImpl {
    async fn login_with_token(&self, user_id: &UserId, token: &str, refresh_token: Option<String>, disable_ssl: bool) -> Result<Client, TachyonError> {
        let device_id = get_device_id(user_id)?;
        let store_path = get_store_path(user_id);

        let client = get_matrix_client_builder(user_id.server_name(), None, disable_ssl)
            .sqlite_store(store_path, None)
            .build()
            .await?;

        client.restore_session(AuthSession::Matrix(MatrixSession {
            meta: matrix_sdk::SessionMeta { user_id: user_id.to_owned(), device_id },
            tokens: SessionTokens { access_token: token.to_string(), refresh_token },
        })).await?;

        client.whoami().await?;
        Ok(client)
    }


    async fn login_with_password(&self, matrix_id: &UserId, password: &str, disable_ssl: bool) -> Result<(AccessToken, Client), TachyonError> {
        create_dir(get_user_data(matrix_id).as_path());

        let client = get_matrix_client_builder(matrix_id.server_name(), None, disable_ssl).build().await?;
        let device_id = get_device_id(matrix_id).ok();


        let mut login_builder = client.matrix_auth()
            .login_username(&matrix_id, password)
            .initial_device_display_name("Windows Live Messenger (Tachyon)");


        if let Some(device_id) = device_id.as_ref() {
            login_builder = login_builder
                .device_id(device_id.to_string().as_str())
        }

        let login_result = login_builder.send().await?;

        if device_id.is_none() {
           if let Err(e) = store_device_id(&login_result.user_id, &login_result.device_id) {
               error!("Fatal: Could not persist device Id, logging out... cause: {}", e);
               client.logout().await?;
           }
        }

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
        Ok((login_result.access_token, client))
    }
}

fn store_device_id(user_id: &UserId, device_id: &DeviceId) -> Result<(), anyhow::Error> {
    let device_id_file = get_user_data(user_id).join(".device_id");
    fs::write(device_id_file, device_id.to_string().as_bytes()).map_err(|e| anyhow!(e))
}

fn get_device_id(user_id: &UserId) -> Result<OwnedDeviceId, anyhow::Error> {
    let device_id_file = get_user_data(user_id).join(".device_id");
    let raw_device_id = fs::read_to_string(device_id_file).map_err(|e| anyhow!(e))?;
    Ok(OwnedDeviceId::try_from(raw_device_id)?)
}

fn get_device_display_name(device_id: &DeviceId) -> String {
    format!("Tachyon-{}", &device_id)
}

fn get_matrix_client_builder(server_name: &ServerName, homeserver_url: Option<String>, disable_ssl: bool) -> ClientBuilder {
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
            client_builder = client_builder.homeserver_url(&homeserver_url)
        }
    }

    client_builder
}