use anyhow::anyhow;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use reqwest::Url;
use std::str::FromStr;

use crate::matrix::login::login_with_password;
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::tachyon::mappers::uuid::ToUuid;
use crate::web::soap::error::RST2Error;
use crate::web::soap::shared;

pub const MAGIC_PASSWORD: &str ="tachyon";

pub async fn rst2_handler(headers: HeaderMap, State(state): State<GlobalState>, body: String) -> Result<Response, RST2Error> {


     let request = RST2RequestMessageSoapEnvelope::try_from_xml(&body)?;

    let creds = request.header.security.username_token
        .ok_or(RST2Error::AuthenticationFailed { source: anyhow!("Request Security Header didn't contain credentials") })?;

    let email = EmailAddress::from_str(&creds.username)?;

    let matrix_id = email.to_owned_user_id();

    if &creds.password == MAGIC_PASSWORD {
        return match state.take_pending_ticket(&email) {
            None => {
                url_open::open(&Url::from_str(format!("http://127.0.0.1:{}/tachyon/auth?username={}", state.get_config().http_port, email.as_str()).as_str()).unwrap());
                Err(RST2Error::InternalServerError { source: anyhow!("Used magic password, opening web login.") })
            }
            Some(ticket_token) => {
                let soap_body = RST2ResponseFactory::get_rst2_success_response(
                    ticket_token,
                    email.to_string(),
                    email.to_uuid(),
                );

                Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
            }
        };
    }

    let (matrix_token, _client) = login_with_password(matrix_id, &creds.password, !state.get_config().strict_ssl).await?;

    let ticket_token = TicketToken(state.secret_encryptor().encrypt(&matrix_token)
        .map_err(|e| RST2Error::InternalServerError { source: anyhow!("Failed to encrypt token: {}", e) })?
    );

     let soap_body = RST2ResponseFactory::get_rst2_success_response(
         ticket_token,
         email.to_string(),
        email.to_uuid(),
     );

     Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}



