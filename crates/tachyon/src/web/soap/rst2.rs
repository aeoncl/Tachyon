use std::str::FromStr;

use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use msnp::soap::traits::xml::{ToXml, TryFromXml};

use crate::matrix::login::login_with_password;
use crate::tachyon::identifiers::MatrixIdCompatible;
use crate::tachyon::traits::{ToUuid};
use crate::web::soap::error::RST2Error;
use crate::web::soap::shared;

pub async fn rst2_handler(body: String) -> Result<Response, RST2Error> {

     let request = RST2RequestMessageSoapEnvelope::try_from_xml(&body)?;

    let creds = request.header.security.username_token.ok_or(RST2Error::AuthenticationFailed { source: anyhow!("Request Security Header didn't contain credentials") })?;

    let email = EmailAddress::from_str(&creds.username)?;

    let matrix_id = email.to_owned_user_id();

    let (token, _client) = login_with_password(matrix_id, &creds.password, false).await?;

     let soap_body = RST2ResponseFactory::get_rst2_success_response(
         TicketToken(token),
         creds.username,
        email.to_uuid(),
     );

     Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}



