use std::path::Path;
use anyhow::anyhow;
use log::info;
use matrix_sdk::{AuthSession, Client, ClientBuilder, ServerName};
use matrix_sdk::matrix_auth::{MatrixSession, MatrixSessionTokens};
use matrix_sdk::ruma::{device_id, OwnedUserId};
use crate::models::tachyon_error::TachyonError;
use crate::utils::identifiers::get_matrix_device_id;

pub fn get_matrix_client_builder(server_name: &ServerName, homeserver_url: Option<String>, disable_ssl: bool) -> ClientBuilder {
    let mut client_builder = Client::builder();

    if disable_ssl {
        client_builder = client_builder.disable_ssl_verification();
    }

    if homeserver_url.is_none() {
        client_builder = client_builder.server_name(server_name)
    } else {
        info!("Setting Homeserver on the client");
        client_builder = client_builder.homeserver_url(&homeserver_url.as_ref().unwrap())
    }

    return client_builder;
}

pub async fn login(matrix_id: OwnedUserId, token: String, store_path: &Path, homeserver_url: Option<String>, disable_ssl: bool) -> Result<Client, TachyonError> {
    let device_id = get_matrix_device_id();
    let device_id = device_id!(device_id.as_str()).to_owned();

    let client =  get_matrix_client_builder(matrix_id.server_name(), homeserver_url, disable_ssl)
        .sqlite_store(store_path, None)
        .build()
        .await?;

    client.restore_session(AuthSession::Matrix(MatrixSession {
        meta: matrix_sdk::SessionMeta { user_id: matrix_id, device_id },
        tokens: MatrixSessionTokens { access_token: token, refresh_token: None },
    }))
        .await.map_err(|e| TachyonError::AuthenticationError{sauce: anyhow!(e).context("Restore session failed")})?;

    client.whoami().await.map_err(|e| TachyonError::AuthenticationError{sauce: anyhow!(e).context("Call to whoami() failed")})?;
    return Ok(client);
}