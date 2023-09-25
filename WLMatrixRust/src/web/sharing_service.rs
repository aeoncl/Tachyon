use std::{str::{from_utf8, FromStr}};

use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, post, web};
use http::{header::HeaderName, StatusCode};
use log::{info, warn};
use substring::Substring;
use yaserde::{de::from_str, ser::to_string};

use crate::{AB_LOCATOR, generated::{msnab_datatypes::types::{BaseMember, RoleId}, msnab_sharingservice::{bindings::{FindMembershipMessageSoapEnvelope, FindMembershipResponseMessageSoapEnvelope}, factories::FindMembershipResponseFactory}}, models::abch::events::AddressBookEvent, MSN_CLIENT_LOCATOR, repositories::repository::Repository, web::{error::WebError, webserver::DEFAULT_CACHE_KEY}};

#[post("/abservice/SharingService.asmx")]
pub async fn soap_sharing_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    if let Some(soap_action_header) = request
        .headers()
        .get(HeaderName::from_str("SOAPAction").unwrap())
    {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()) {
            let name = soap_action.split("/").last().unwrap_or(soap_action);
            info!("{}Request: {}", &name, from_utf8(&body)?);

            match soap_action {
                "http://www.msn.com/webservices/AddressBook/FindMembership" => {
                    return ab_sharing_find_membership(body, request).await;
                },
                _ => {}
            }
        } else {
            info!("SharingService UnknownRequest: {}", from_utf8(&body)?);
        }
    }
    return Ok(HttpResponseBuilder::new(StatusCode::NOT_FOUND)
        .append_header(("Content-Type", "application/soap+xml"))
        .finish());
}

async fn ab_sharing_find_membership(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body).unwrap();

    let request = from_str::<FindMembershipMessageSoapEnvelope>(body)?;
    let deltas_only = request.body.body.find_membership_request.deltas_only;
    
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let ticket_token = &header.ab_auth_header.ticket_token;
    let matrix_token = header.ab_auth_header.ticket_token.substring(2, ticket_token.len()).to_string();

    let cache_key = &header.application_header.cache_key .unwrap_or(DEFAULT_CACHE_KEY.to_string());

    let msn_client = MSN_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let me = msn_client.get_user();
    let response: FindMembershipResponseMessageSoapEnvelope;
    
    if header.application_header.partner_scenario.as_str() == "Initial" {
        response = FindMembershipResponseFactory::get_empty_response(
            me.get_uuid(),
            me.get_msn_addr(),
            cache_key.clone(), deltas_only);
    } else {

        let membership_events = AB_LOCATOR.get_membership_events(&matrix_token).await.unwrap();
        let (allow_list, reverse_list, block_list, pending_list) = get_messenger_service(&membership_events);
        let msg_service = FindMembershipResponseFactory::get_messenger_service(allow_list, block_list, reverse_list, pending_list, false);

        //Check if we need to use the UUID from the client here ?? seems important for dedup in the contact folder !
        response = FindMembershipResponseFactory::get_response(
            me.get_uuid(),
            me.get_msn_addr(),
            cache_key.clone(), msg_service);
    }

    let response_serialized = to_string(&response)?;
    info!("find_membership_response: {}", response_serialized);
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

fn get_messenger_service(membership_events: &Vec<AddressBookEvent>) -> (Vec<BaseMember>, Vec<BaseMember>, Vec<BaseMember>, Vec<BaseMember>) {
    let mut allow_list = Vec::new();
    let mut reverse_list = Vec::new();
    let mut block_list = Vec::new();
    let mut pending_list = Vec::new();

    for ev in membership_events {
        if let AddressBookEvent::MembershipEvent(content) = ev {
            match &content.list {
                &RoleId::Allow => {
                    allow_list.push(content.member.clone())
                },
                &RoleId::Reverse => {
                    reverse_list.push(content.member.clone())
                },
                &RoleId::Block => {
                    block_list.push(content.member.clone())
                },
                &RoleId::Pending => {
                    pending_list.push(content.member.clone())
                },
                _ => {
                    warn!("Address Book event contained forbidden list: {}", &content.list)
                }
            }
        }
    }
    return (allow_list, reverse_list, block_list, pending_list);
}