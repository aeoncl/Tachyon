use std::str::FromStr;

use anyhow::anyhow;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum::routing::delete;

use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::rsi::delete_messages::request::DeleteMessagesSoapEnvelope;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use msnp::soap::rsi::get_metadata::request::GetMetadataMessageSoapEnvelope;
use msnp::soap::rsi::service_header::RSIAuthSoapEnvelope;
use msnp::soap::traits::xml::TryFromXml;

use crate::notification::client_store::ClientStoreFacade;
use crate::web::soap::error::ABError;
use crate::web::soap::rsi::delete_messages::delete_messages;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::rsi::get_message::get_message;
use crate::web::soap::rsi::get_metadata::get_metadata;

pub async fn rsi(headers: HeaderMap, State(state): State<ClientStoreFacade>, body: String) -> Result<Response, RSIError> {


    let soap_action = headers.get("SOAPAction").ok_or(RSIError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = RSIAuthSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken(header_env.header.ok_or(RSIError::AuthenticationFailed {source: anyhow!("Missing Soap Header", ), service_url: "https://rsi.hotmail.com/rsi/rsi.asmx".to_string() })?.passport_cookie.t);

    let mut client_data = state.get_client_data(&token.0).ok_or(RSIError::AuthenticationFailed {source: anyhow!("Missing Client Data in client store"), service_url: "https://rsi.hotmail.com/rsi/rsi.asmx".to_string() })?;

    let client = client_data.get_matrix_client();

    let client_token = client.access_token().ok_or(RSIError::AuthenticationFailed {source: anyhow!("No Token present in Matrix Client"), service_url: "https://rsi.hotmail.com/rsi/rsi.asmx".to_string() })?;
    if token != client_token {
        return Err(RSIError::AuthenticationFailed { source: anyhow!("Supplied Token & Matrix Token don't match: {} == {}", &token.0, &client_token), service_url: "https://rsi.hotmail.com/rsi/rsi.asmx".to_string() });
    }

    match soap_action {

        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/GetMessage" => {
            get_message(GetMessageMessageSoapEnvelope::try_from_xml(&body)?, token, client, &mut client_data).await
        },
        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/GetMetadata" => {
            get_metadata(GetMetadataMessageSoapEnvelope::try_from_xml(&body)?, token, client, &mut client_data).await

        },
        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/DeleteMessages" => {
            delete_messages(DeleteMessagesSoapEnvelope::try_from_xml(&body)?, token, client, &mut client_data).await
        },
        _ => {
            Err(RSIError::UnsupportedSoapAction(soap_action.to_string()))
        }
    }

}
