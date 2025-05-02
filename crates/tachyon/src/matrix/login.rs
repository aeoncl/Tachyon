use std::path::{Path, PathBuf};
use anyhow::anyhow;
use log::debug;

use matrix_sdk::{AuthSession, Client, ClientBuilder, ServerName, SessionTokens};
use matrix_sdk::authentication::matrix::MatrixSession;
use matrix_sdk::ruma::{device_id, OwnedUserId, UserId};

use msnp::shared::models::ticket_token::TicketToken;

use crate::shared::paths;
use crate::shared::error::{MatrixConversionError, TachyonError};
use crate::shared::identifiers::MatrixDeviceId;
use crate::shared::paths::get_store_path;
use crate::web::soap::error::RST2Error;

fn get_device_id() -> Result<MatrixDeviceId, MatrixConversionError> {
    MatrixDeviceId::from_hostname()
}

pub fn get_device_display_name(device_id: &MatrixDeviceId) -> String {
    format!("Tachyon-{}", &device_id)
}



pub fn get_matrix_client_builder(server_name: &ServerName, homeserver_url: Option<String>, disable_ssl: bool) -> ClientBuilder {
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

pub async fn login_with_token(matrix_id: OwnedUserId, token: TicketToken, disable_ssl: bool) -> Result<Client, TachyonError> {
    let device_id_str = get_device_id()?.to_string();
    let device_id = device_id!(device_id_str.as_str()).to_owned();


    let store_path = get_store_path(&matrix_id).ok_or(anyhow!("Couldn't get store path"))?;
    debug!("storepath: {:?}", &store_path);

    let client = get_matrix_client_builder(matrix_id.server_name(), None, disable_ssl)
        .sqlite_store(store_path, None)
        .build()
        .await?;

    client.restore_session(AuthSession::Matrix(MatrixSession {
        meta: matrix_sdk::SessionMeta { user_id: matrix_id, device_id },
        tokens: SessionTokens { access_token: token.0, refresh_token: None },
    })).await?;

    client.whoami().await?;
    Ok(client)
}

pub async fn login_with_password(matrix_id: OwnedUserId, password: &str, disable_ssl: bool) -> Result<(String, Client), TachyonError> {
    let client = get_matrix_client_builder(matrix_id.server_name(), None, true).build().await?;

    let device_id = get_device_id()?;
    let device_id_as_str = device_id.to_string();

    let result = client.matrix_auth()
        .login_username(&matrix_id, password)
        .device_id(&device_id_as_str)
        .initial_device_display_name(get_device_display_name(&device_id).as_str())
        .send()
        .await?;

    Ok((result.access_token, client))
}
