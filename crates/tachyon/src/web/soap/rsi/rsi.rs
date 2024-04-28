use std::str::FromStr;

use anyhow::anyhow;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use msnp::soap::rsi::service_header::RSIAuthSoapEnvelope;
use msnp::soap::traits::xml::TryFromXml;

use crate::notification::client_store::ClientStoreFacade;
use crate::web::soap::error::ABError;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::rsi::get_message::get_message;

pub async fn rsi(headers: HeaderMap, State(state): State<ClientStoreFacade>, body: String) -> Result<Response, RSIError> {


    let soap_action = headers.get("SOAPAction").ok_or(RSIError::MissingHeader("SOAPAction".into()))?.to_str()?;

    let header_env = RSIAuthSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken(header_env.header.ok_or(RSIError::AuthenticationFailed {source: anyhow!("Missing Soap Header")})?.passport_cookie.t);

    let client_data = state.get_client_data(&token.0).ok_or(RSIError::AuthenticationFailed {source: anyhow!("Missing Client Data in client store")})?;

    let client = client_data.get_matrix_client();

    let client_token = client.access_token().ok_or(RSIError::AuthenticationFailed {source: anyhow!("No Token present in Matrix Client")})?;
    if token != client_token {
        return Err(RSIError::AuthenticationFailed { source: anyhow!("Supplied Token & Matrix Token don't match: {} == {}", &token.0, &client_token) });
    }

    match soap_action {

        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/GetMessage" => {
            get_message(GetMessageMessageSoapEnvelope::try_from_xml(&body)?, token, client, &state).await
        },
        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/GetMetadata" => {
            todo!()
        },
        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/DeleteMessages" => {
            todo!()
        },
        _ => {
            todo!()
        }
    }

}
