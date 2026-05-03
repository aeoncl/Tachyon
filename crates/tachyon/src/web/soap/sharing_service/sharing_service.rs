use crate::tachyon::global_state::GlobalState;
use crate::web::soap::error::ABError;
use crate::web::soap::sharing_service::add_member::add_member;
use crate::web::soap::sharing_service::delete_member::delete_member;
use crate::web::soap::sharing_service::find_membership::find_membership;
use anyhow::anyhow;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;
use log::error;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::request_header::AuthHeaderSoapEnvelope;
use msnp::soap::abch::sharing_service::add_member::request::AddMemberMessageSoapEnvelope;
use msnp::soap::abch::sharing_service::delete_member::request::DeleteMemberMessageSoapEnvelope;
use msnp::soap::abch::sharing_service::find_membership::request::FindMembershipRequestSoapEnvelope;
use msnp::soap::traits::xml::TryFromXml;
use std::str::FromStr;
use crate::tachyon::repository::RepositoryStr;

pub async fn sharing_service(headers: HeaderMap, State(state): State<GlobalState>, body: String) -> Result<Response, ABError> {

    let soap_action = headers.get("SOAPAction").ok_or(ABError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = AuthHeaderSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken::from_str(&header_env.header.ab_auth_header.ticket_token).unwrap();


    let tachyon_client = state.tachyon_clients().get(token.as_str()).ok_or(ABError::AuthenticationFailed {source: anyhow!("Expected Tachyon Client to be present in client Store")})?;

    match soap_action {
        "http://www.msn.com/webservices/AddressBook/FindMembership" => {
            find_membership(FindMembershipRequestSoapEnvelope::try_from_xml(&body)?, token, tachyon_client).await
        },
        "http://www.msn.com/webservices/AddressBook/AddMember" => {
            add_member(AddMemberMessageSoapEnvelope::try_from_xml(&body)?, token, tachyon_client).await
        },
        "http://www.msn.com/webservices/AddressBook/DeleteMember" => {
            delete_member(DeleteMemberMessageSoapEnvelope::try_from_xml(&body)?, token, tachyon_client).await
        },
        _ => {
            error!("SOAP|ABCH: Unsupported soap action: {}", &soap_action);
            Err(ABError::UnsupportedSoapAction(soap_action.to_string()))
        }
    }
}