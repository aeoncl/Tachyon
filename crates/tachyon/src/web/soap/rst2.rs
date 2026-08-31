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

/// Issues the ticket the client will present on `USR`.
///
/// The client calls this on every startup with the password it saved, and that password is
/// not checked: Tachyon has no password of its own, and the real authentication happens
/// during the `USR` exchange, where the client can be walked through an interactive login.
/// The ticket only names an account so a backend session can be found or created — see
/// `tachyon::identifiers::ticket`.
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
