use std::str::FromStr;
use std::time::Duration;
use anyhow::anyhow;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use tokio::time::sleep;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use msnp::soap::traits::xml::{ToXml, TryFromXml};

use crate::matrix::login::login_with_password;
use crate::tachyon::identifiers::MatrixIdCompatible;
use crate::tachyon::tachyon_state::TachyonState;
use crate::tachyon::traits::{ToUuid};
use crate::web::soap::error::RST2Error;
use crate::web::soap::shared;

pub async fn rst2_handler(headers: HeaderMap, State(state): State<TachyonState>, body: String) -> Result<Response, RST2Error> {

     let request = RST2RequestMessageSoapEnvelope::try_from_xml(&body)?;

    let creds = request.header.security.username_token
        .ok_or(RST2Error::AuthenticationFailed { source: anyhow!("Request Security Header didn't contain credentials") })?;

    let email = EmailAddress::from_str(&creds.username)?;

    let matrix_id = email.to_owned_user_id();

    let (matrix_token, _client) = login_with_password(matrix_id, &creds.password, true).await?;

    let ticket_token = TicketToken(state.secret_encryptor().encrypt(&matrix_token)
        .map_err(|e| RST2Error::InternalServerError { source: anyhow!("Failed to encrypt token: {}", e) })?
    );

     let soap_body = RST2ResponseFactory::get_rst2_success_response(
         ticket_token,
         creds.username,
        email.to_uuid(),
     );

     Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}



