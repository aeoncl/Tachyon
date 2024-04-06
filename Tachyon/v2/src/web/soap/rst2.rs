use std::str::FromStr;
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::Request;
use axum::handler::Handler;
use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{Client, ClientBuilder, ServerName};
use msnp::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use log::{debug, error, info};
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use crate::shared::error::MatrixConversionError;
use crate::shared::identifiers::MatrixDeviceId;
use crate::shared::traits::{ToUuid, TryFromMsnAddr};
use crate::web::soap::error::{RST2Error, SoapError};
use crate::web::soap::error::SoapError::AuthenticationFailed;
use crate::web::soap::shared;

pub async fn rst2_handler(body: String) -> Result<Response, RST2Error> {

     let request = RST2RequestMessageSoapEnvelope::try_from_xml(&body)?;

    let creds = request.header.security.username_token.ok_or(RST2Error::AuthenticationFailed { source: anyhow!("Request Security Header didn't contain credentials") })?;

    let matrix_id = OwnedUserId::try_from_msn_addr(&creds.username)?;

    //TODO remove this hardcoded url
    let client = get_matrix_client_builder(matrix_id.server_name(), None, true).build().await.map_err(|e| RST2Error::InternalServerError {source: e.into()})?;

    let device_id = MatrixDeviceId::from_hostname()?.to_string();

    let result = client.matrix_auth()
                .login_username(&matrix_id, &creds.password)
                .device_id(&device_id)
                .initial_device_display_name("Tachyon")
                .send()
                .await
                .map_err(|e| RST2Error::AuthenticationFailed{ source: e.into() })?;

     let soap_body = RST2ResponseFactory::get_rst2_success_response(
         TicketToken(result.access_token),
         creds.username,
        matrix_id.to_uuid(),
     );

     Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}


impl IntoResponse for RST2Error {
    fn into_response(self) -> Response {
        error!("SOAP|RST2: {:?}", &self);
        match self {
            RST2Error::AuthenticationFailed { .. } => {
                shared::build_soap_response(RST2ResponseFactory::get_auth_error_response(), StatusCode::OK)
            },
            RST2Error::SoapMarshallError(_) => {
                shared::build_soap_response(RST2ResponseFactory::get_bad_request(), StatusCode::OK)
            }
            RST2Error::MatrixConversionError(_) => {
                shared::build_soap_response(RST2ResponseFactory::get_bad_request(), StatusCode::OK)
            }
            RST2Error::InternalServerError { .. } => {
                shared::build_soap_response(RST2ResponseFactory::get_bad_request(), StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
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

    return client_builder;
}