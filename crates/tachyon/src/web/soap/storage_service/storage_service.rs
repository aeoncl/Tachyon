use std::str::FromStr;
use anyhow::anyhow;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum_macros::debug_handler;
use log::error;
use matrix_sdk::Client;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::request_header::AuthHeaderSoapEnvelope;
use msnp::soap::storage_service::get_profile::request::GetProfileMessageSoapEnvelope;
use msnp::soap::storage_service::get_profile::response::GetProfileResponseMessageSoapEnvelope;
use msnp::soap::storage_service::headers::StorageServiceRequestSoapEnvelope;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use crate::notification::client_store::ClientStoreFacade;
use crate::shared::identifiers::MatrixIdCompatible;
use crate::shared::traits::ToUuid;
use crate::web::soap::error::ABError;
use crate::web::soap::shared;
use crate::web::web_endpoints::DEFAULT_CACHE_KEY;

pub async fn storage_service(headers: HeaderMap, State(state): State<ClientStoreFacade>, body: String) -> Result<Response, ABError> {

    let soap_action = headers.get("SOAPAction").ok_or(ABError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = StorageServiceRequestSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken::from_str(&header_env.header.storage_user.unwrap().ticket_token).unwrap();

    let client_data = state.get_client_data(&token.0).ok_or(ABError::AuthenticationFailed {source: anyhow!("Expected Client Data to be present in client Store")})?;

    let client = client_data.get_matrix_client();

    let client_token = client.access_token().ok_or(ABError::AuthenticationFailed {source: anyhow!("No Token present in Matrix Client")})?;
    if token != client_token {
        return Err(ABError::AuthenticationFailed { source: anyhow!("Supplied Token & Matrix Token don't match: {} == {}", &token.0, &client_token) });
    }

    match soap_action {
        "http://www.msn.com/webservices/storage/2008/GetProfile" => {
            get_profile(GetProfileMessageSoapEnvelope::try_from_xml(&body)?, token, client).await
        },
        _ => {
            error!("SOAP|ABCH: Unsupported soap action: {}", &soap_action);
            Err(ABError::UnsupportedSoapAction(soap_action.to_string()))
        }
    }


}

async fn get_profile(request: GetProfileMessageSoapEnvelope, token: TicketToken, matrix_client: Client) -> Result<Response, ABError> {
    let user_id = matrix_client.user_id().ok_or(anyhow!("Expected to have user_id in matrix client"))?;
    let msn_addr = EmailAddress::from_user_id(user_id);
    let uuid = msn_addr.to_uuid();

    let display_name = matrix_client.account().get_display_name().await?.unwrap_or(msn_addr.to_string());

    //TODO fetch image
    let soap_body = GetProfileResponseMessageSoapEnvelope::new(uuid, DEFAULT_CACHE_KEY.to_string(), display_name, String::new(), None);
    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

}