use anyhow::anyhow;
use axum::extract::State;
use axum::http::header::USER_AGENT;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use msnp::shared::models::client_version::ClientVersion;
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
/// The client calls this endpoint after every reboot, otherwise it uses its persisted ticket
/// token with the USR command. This endpoint doesn't check any credentials: the ticket is
/// core's token for this client of the address, so the USR flow can relink the connection to
/// the backend session that token names. The client version comes from the `User-Agent`,
/// the one place this request names the client.
pub async fn rst2_handler(
    headers: HeaderMap,
    State(state): State<GlobalState>,
    body: String,
) -> Result<Response, RST2Error> {
    let request = RST2RequestMessageSoapEnvelope::try_from_xml(&body)?;

    let creds = request.header.security.username_token.ok_or(
        RST2Error::AuthenticationFailed {
            source: anyhow!("Request Security Header didn't contain credentials"),
        },
    )?;

    let user_agent = headers
        .get(USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let client = ClientVersion::from_user_agent(user_agent).ok_or_else(|| {
        RST2Error::AuthenticationFailed {
            source: anyhow!("The request does not name the client: {user_agent}"),
        }
    })?;

    let email = EmailAddress::from_str(&creds.username)?;
    let ticket_token = state.ticket_for(&email, &client);

    let soap_body = RST2ResponseFactory::get_rst2_success_response(
        ticket_token,
        email.to_string(),
        email.to_uuid(),
    );

    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}
