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
use msnp::shared::models::msn_user::MSNUser;
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
use msnp::soap::abch::request_header::AuthHeaderSoapEnvelope;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use crate::matrix::direct_target_resolver::resolve_direct_target;
use crate::notification::client_store::ClientStoreFacade;
use crate::shared::identifiers::MatrixIdCompatible;
use crate::web::soap::error::ABError;
use crate::web::soap::error::ABError::InternalServerError;
use crate::web::soap::shared;
use crate::web::soap::shared::build_soap_response;

impl IntoResponse for ABError {
    fn into_response(self) -> Response {
        build_soap_response("".into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}

#[debug_handler]
pub async fn address_book_service(headers: HeaderMap, State(state): State<ClientStoreFacade>, body: String) -> Result<Response, ABError> {

    let soap_action = headers.get("SOAPAction").ok_or(ABError::MissingHeader("SOAPAction".into()))?.to_str()?;

    let header_env = AuthHeaderSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken::from_str(&header_env.header.ab_auth_header.ticket_token).unwrap();

    let client = state.get_matrix_client(&token.0).await?.ok_or(ABError::AuthenticationFailed {source: anyhow!("Missing Matrix Client in client store")})?;

    let client_token = client.access_token().ok_or(ABError::AuthenticationFailed {source: anyhow!("No Token present in Matrix Client")})?;
    if token != client_token {
        return Err(ABError::AuthenticationFailed { source: anyhow!("Supplied Token & Matrix Token don't match: {} == {}", &token.0, &client_token) });
    }

    match soap_action {
        "http://www.msn.com/webservices/AddressBook/ABFindContactsPaged" => {
            ab_find_contacts_paged(AbfindContactsPagedMessageSoapEnvelope::try_from_xml(&body)?, token, client).await
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

async fn ab_find_contacts_paged(request : AbfindContactsPagedMessageSoapEnvelope, token: TicketToken, client: Client) -> Result<Response, ABError> {
    let body = request.body.body;
    let cache_key = request.header.expect("to be here").application_header.cache_key.unwrap_or_default();
    let user_id = client.user_id().ok_or(InternalServerError { source: anyhow!("Matrix client has no user ID.") })?;
    let msn_addr = <EmailAddress as MatrixIdCompatible>::from(user_id).to_string();

    if body.filter_options.deltas_only {
        // Only incremental changes
        let contacts = get_fullsync_contact_list(&client).await?;

        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new(Uuid::from_seed(&user_id.to_string()), &cache_key, &msn_addr, &msn_addr, contacts, false);

        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))


    } else {
        // Full contact list demanded.
        todo!()
    }
}

async fn get_fullsync_contact_list(matrix_client: &Client) -> Result<Vec<ContactType>, matrix_sdk::Error> {
    let mut out = Vec::new();

    let me = matrix_client.user_id().expect("A user to be logged in when fetching fullsync");

    for joined_room in matrix_client.joined_rooms() {
        if joined_room.is_direct().await? {
            let direct_target = resolve_direct_target(&joined_room.direct_targets(), &joined_room, me, matrix_client).await;
            if let Some(direct_target) = direct_target {

                if let Some(member) = joined_room.get_member(&direct_target).await? {
                    let target_usr = MSNUser::with_email_addr(EmailAddress::from_owned(direct_target));
                    let target_uuid = target_usr.uuid;
                    let target_msn_addr = target_usr.endpoint_id.email_addr.to_string();

                    match member.membership() {
                        MembershipState::Invite => {
                            let contact = ContactType::new(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::LivePending, false);
                            out.push(contact);
                        }
                        _ => {
                            let contact = ContactType::new(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, false);
                            out.push(contact);
                        }
                    }
                }
            } else {
                info!("Fullsync Fetch: No direct target found for room: {}", &joined_room.room_id());
            }

        }
    }

    return Ok(out);
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