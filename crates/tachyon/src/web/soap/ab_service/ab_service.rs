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
use crate::tachyon::global::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::soap::ab_service::ab_contact_add::ab_contact_add;
use crate::web::soap::ab_service::ab_contact_delete::ab_contact_delete;
use crate::web::soap::ab_service::ab_contact_update::ab_contact_update;
use crate::web::soap::ab_service::ab_find_contacts_paged::ab_find_contacts_paged;
use crate::web::soap::error::ABError;
#[debug_handler]
pub async fn address_book_service(headers: HeaderMap, State(state): State<GlobalState>, body: String) -> Result<Response, ABError> {

    let soap_action = headers.get("SOAPAction").ok_or(ABError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = AuthHeaderSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken::from_str(&header_env.header.ab_auth_header.ticket_token).unwrap();

    let tachyon_client = state.tachyon_clients().get(token.as_str()).ok_or(ABError::AuthenticationFailed {source: anyhow!("Expected Tachyon Client to be present in client Store")})?;

    match soap_action {
        "http://www.msn.com/webservices/AddressBook/ABFindContactsPaged" => {
            ab_find_contacts_paged(AbfindContactsPagedMessageSoapEnvelope::try_from_xml(&body)?, token, tachyon_client).await
        },
        "http://www.msn.com/webservices/AddressBook/ABContactAdd" => {
            ab_contact_add(AbcontactAddMessageSoapEnvelope::try_from_xml(&body)?, token, tachyon_client, &soap_action).await
        },
        "http://www.msn.com/webservices/AddressBook/ABContactDelete" => {
            ab_contact_delete(AbcontactDeleteMessageSoapEnvelope::try_from_xml(&body)?, token, tachyon_client, &soap_action).await

        },
        "http://www.msn.com/webservices/AddressBook/ABContactUpdate" => {
            ab_contact_update(AbcontactUpdateMessageSoapEnvelope::try_from_xml(&body)?, token, tachyon_client, &soap_action).await

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

async fn ab_group_add(_request : AbgroupAddMessageSoapEnvelope, _token: TicketToken) -> Result<Response, ABError> {
    todo!()
}