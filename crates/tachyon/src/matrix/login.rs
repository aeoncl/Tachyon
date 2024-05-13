use std::path::Path;

use matrix_sdk::{AuthSession, Client, ClientBuilder, Error, ServerName};
use matrix_sdk::matrix_auth::{MatrixSession, MatrixSessionTokens};
use matrix_sdk::ruma::{device_id, OwnedUserId};
use msnp::shared::models::ticket_token::TicketToken;

use crate::shared::identifiers::MatrixDeviceId;


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

pub async fn login(matrix_id: OwnedUserId, device_id: String, token: TicketToken, store_path: &Path, homeserver_url: Option<String>, disable_ssl: bool) -> Result<Client, Error> {
    let device_id_str = device_id.to_string();
    let device_id = device_id!(device_id_str.as_str()).to_owned();

    let client = get_matrix_client_builder(matrix_id.server_name(), homeserver_url, disable_ssl)
        .sqlite_store(store_path, None)
        .build()
        .await.unwrap();

    client.restore_session(AuthSession::Matrix(MatrixSession {
        meta: matrix_sdk::SessionMeta { user_id: matrix_id, device_id },
        tokens: MatrixSessionTokens { access_token: token.0, refresh_token: None },
    })).await?;

    client.whoami().await?;
    Ok(client)
}