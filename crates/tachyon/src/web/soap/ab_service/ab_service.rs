use std::str::FromStr;
use anyhow::anyhow;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use axum_macros::debug_handler;
use log::error;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::ab_service::ab_contact_add::request::AbcontactAddMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_contact_delete::request::AbcontactDeleteMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_contact_update::request::AbcontactUpdateMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_group_add::request::AbgroupAddMessageSoapEnvelope;
use msnp::soap::abch::request_header::AuthHeaderSoapEnvelope;
use msnp::soap::traits::xml::TryFromXml;
use crate::notification::client_store::ClientStoreFacade;
use crate::web::soap::ab_service::ab_find_contacts_paged::ab_find_contacts_paged;
use crate::web::soap::error::ABError;
#[debug_handler]
pub async fn address_book_service(headers: HeaderMap, State(state): State<ClientStoreFacade>, body: String) -> Result<Response, ABError> {

    let soap_action = headers.get("SOAPAction").ok_or(ABError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = AuthHeaderSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken::from_str(&header_env.header.ab_auth_header.ticket_token).unwrap();

    let client_data = state.get_client_data(&token.0).ok_or(ABError::AuthenticationFailed {source: anyhow!("Expected Client Data to be present in client Store")})?;

    let client = client_data.matrix_client();

    let client_token = client.access_token().ok_or(ABError::AuthenticationFailed {source: anyhow!("No Token present in Matrix Client")})?;
    if token != client_token {
        return Err(ABError::AuthenticationFailed { source: anyhow!("Supplied Token & Matrix Token don't match: {} == {}", &token.0, &client_token) });
    }

    match soap_action {
        "http://www.msn.com/webservices/AddressBook/ABFindContactsPaged" => {
            ab_find_contacts_paged(AbfindContactsPagedMessageSoapEnvelope::try_from_xml(&body)?, token, client, client_data).await
        },
        "http://www.msn.com/webservices/AddressBook/ABContactAdd" => {
            ab_contact_add(AbcontactAddMessageSoapEnvelope::try_from_xml(&body)?, token).await
        },
        "http://www.msn.com/webservices/AddressBook/ABContactDelete" => {
            ab_contact_delete(AbcontactDeleteMessageSoapEnvelope::try_from_xml(&body)?, token).await

        },
        "http://www.msn.com/webservices/AddressBook/ABContactUpdate" => {
            ab_contact_update(AbcontactUpdateMessageSoapEnvelope::try_from_xml(&body)?, token).await

        },
        "http://www.msn.com/webservices/AddressBook/ABGroupAdd" => {
            ab_group_add(AbgroupAddMessageSoapEnvelope::try_from_xml(&body)?, token).await
        },
        _ => {
            error!("SOAP|ABCH: Unsupported soap action: {}", &soap_action);
            Err(ABError::UnsupportedSoapAction(soap_action.to_string()))
        }
    }
}



async fn ab_contact_add(_request : AbcontactAddMessageSoapEnvelope, _token: TicketToken) -> Result<Response, ABError> {
    todo!()
}

async fn ab_contact_delete(_request : AbcontactDeleteMessageSoapEnvelope, _token: TicketToken) -> Result<Response, ABError> {
    todo!()
}

async fn ab_contact_update(_request : AbcontactUpdateMessageSoapEnvelope, _token: TicketToken) -> Result<Response, ABError> {
    todo!()
}

async fn ab_group_add(_request : AbgroupAddMessageSoapEnvelope, _token: TicketToken) -> Result<Response, ABError> {
    todo!()
}