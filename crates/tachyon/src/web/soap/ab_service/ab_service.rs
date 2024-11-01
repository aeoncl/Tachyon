use std::str::FromStr;
use std::sync::Arc;
use anyhow::anyhow;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_macros::debug_handler;
use log::{error, info, warn};
use matrix_sdk::{Client, Error};
use matrix_sdk::ruma::events::room::member::MembershipState;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::ab_service::ab_contact_add::request::AbcontactAddMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_contact_delete::request::AbcontactDeleteMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_contact_update::request::AbcontactUpdateMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::response::AbfindContactsPagedResponseMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_group_add::request::AbgroupAddMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_group_contact_add::request::AbgroupContactAddMessageSoapEnvelope;
use msnp::soap::abch::msnab_datatypes::{ContactType, ContactTypeEnum};
use msnp::soap::abch::msnab_faults::SoapFaultResponseEnvelope;
use msnp::soap::abch::request_header::AuthHeaderSoapEnvelope;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use crate::matrix::directs::resolve_direct_target;
use crate::notification::client_store::ClientStoreFacade;
use crate::shared::identifiers::MatrixIdCompatible;
use crate::web::soap::ab_service::ab_find_contacts_paged::ab_find_contacts_paged;
use crate::web::soap::error::ABError;
use crate::web::soap::error::ABError::InternalServerError;
use crate::web::soap::shared;
use crate::web::soap::shared::build_soap_response;
#[debug_handler]
pub async fn address_book_service(headers: HeaderMap, State(state): State<ClientStoreFacade>, body: String) -> Result<Response, ABError> {

    let soap_action = headers.get("SOAPAction").ok_or(ABError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = AuthHeaderSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken::from_str(&header_env.header.ab_auth_header.ticket_token).unwrap();

    let client_data = state.get_client_data(&token.0).ok_or(ABError::AuthenticationFailed {source: anyhow!("Expected Client Data to be present in client Store")})?;

    let client = client_data.get_matrix_client();

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



async fn ab_contact_add(request : AbcontactAddMessageSoapEnvelope, token: TicketToken) -> Result<Response, ABError> {
    todo!()
}

async fn ab_contact_delete(request : AbcontactDeleteMessageSoapEnvelope, token: TicketToken) -> Result<Response, ABError> {
    todo!()
}

async fn ab_contact_update(request : AbcontactUpdateMessageSoapEnvelope, token: TicketToken) -> Result<Response, ABError> {
    todo!()
}

async fn ab_group_add(request : AbgroupAddMessageSoapEnvelope, token: TicketToken) -> Result<Response, ABError> {
    todo!()
}