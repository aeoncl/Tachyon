use anyhow::anyhow;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use msnp::shared::models::email_address::EmailAddress;
use msnp::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use std::str::FromStr;

use crate::tachyon::global_state::GlobalState;
use crate::tachyon::mappers::uuid::ToUuid;
use crate::web::soap::error::RST2Error;
use crate::web::soap::shared;

/// Issues the ticket token the msn client will present with `USR` MSNP Command.
///
/// The client calls this endpoint after every reboot, otherwise it uses it's persisted ticket token with the USR command.
/// This endpoint doesn't check any credentials, the ticket is the user email hashed.
/// It's sole purpose is to allow the USR flow to take place with a predictible ticket that allows us to relink the current BackendSession to a previous one.
pub async fn rst2_handler(
    _headers: HeaderMap,
    State(state): State<GlobalState>,
    body: String,
) -> Result<Response, RST2Error> {
    let request = RST2RequestMessageSoapEnvelope::try_from_xml(&body)?;

    let creds = request.header.security.username_token.ok_or(
        RST2Error::AuthenticationFailed {
            source: anyhow!("Request Security Header didn't contain credentials"),
        },
    )?;

    let email = EmailAddress::from_str(&creds.username)?;
    let ticket_token = state.ticket_for(&email);

    let soap_body = RST2ResponseFactory::get_rst2_success_response(
        ticket_token,
        email.to_string(),
        email.to_uuid(),
    );

    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}
