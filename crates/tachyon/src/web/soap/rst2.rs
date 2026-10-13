use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use std::str::FromStr;

use crate::tachyon::global_state::GlobalState;
use crate::tachyon::identifiers::bridge_link::{BridgeLinkToken, BridgeType, ClientVersion};
use crate::tachyon::mappers::uuid::ToUuid;
use crate::web::soap::error::RST2Error;
use crate::web::soap::shared;

pub async fn rst2_handler(headers: HeaderMap, State(_state): State<GlobalState>, body: String) -> Result<Response, RST2Error> {

    let request = RST2RequestMessageSoapEnvelope::try_from_xml(&body)?;

    let creds = request.header.security.username_token
        .ok_or(RST2Error::AuthenticationFailed { source: anyhow::anyhow!("Request Security Header didn't contain credentials") })?;

    let email = EmailAddress::from_str(&creds.username)?;

    let user_agent = headers
        .get("User-Agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let token = BridgeLinkToken::from_parts(BridgeType::Msn, email.as_str(), ClientVersion::from_user_agent(user_agent));

    // the login page completes the sign-in; the ticket is a stable link, not the secret
    let ticket_token = TicketToken(token.into_string());

    let soap_body = RST2ResponseFactory::get_rst2_success_response(
        ticket_token,
        email.to_string(),
        email.to_uuid(),
    );

    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}
