
use anyhow::anyhow;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::rsi::delete_messages::request::DeleteMessagesSoapEnvelope;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use msnp::soap::rsi::get_metadata::request::GetMetadataMessageSoapEnvelope;
use msnp::soap::rsi::service_header::RSIAuthSoapEnvelope;
use msnp::soap::traits::xml::TryFromXml;

use crate::tachyon::global::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::soap::rsi::delete_messages::delete_messages;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::rsi::get_message::get_message;
use crate::web::soap::rsi::get_metadata::get_metadata;

pub async fn rsi(headers: HeaderMap, State(state): State<GlobalState>, body: String) -> Result<Response, RSIError> {


    let soap_action = headers.get("SOAPAction").ok_or(RSIError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = RSIAuthSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken(header_env.header.ok_or(RSIError::AuthenticationFailed {source: anyhow!("Missing Soap Header", ), service_url: "https://rsi.hotmail.com/rsi/rsi.asmx".to_string() })?.passport_cookie.t);

    let mut tachyon_client = state.tachyon_clients().get(token.as_str()).ok_or(RSIError::AuthenticationFailed {source: anyhow!("Missing Tachyon Client"), service_url: "https://rsi.hotmail.com/rsi/rsi.asmx".to_string() })?;
    let client = tachyon_client.matrix_client().clone();

    match soap_action {

        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/GetMessage" => {
            get_message(GetMessageMessageSoapEnvelope::try_from_xml(&body)?, token, client, &mut tachyon_client).await
        },
        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/GetMetadata" => {
            get_metadata(GetMetadataMessageSoapEnvelope::try_from_xml(&body)?, token, client, &mut tachyon_client).await

        },
        "http://www.hotmail.msn.com/ws/2004/09/oim/rsi/DeleteMessages" => {
            delete_messages(DeleteMessagesSoapEnvelope::try_from_xml(&body)?, token, client, &mut tachyon_client).await
        },
        _ => {
            Err(RSIError::UnsupportedSoapAction(soap_action.to_string()))
        }
    }

}
