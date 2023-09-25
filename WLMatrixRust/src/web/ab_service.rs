use std::{str::{from_utf8, FromStr}};

use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, post, web};
use http::{header::HeaderName, StatusCode};
use log::info;
use substring::Substring;
use yaserde::{de::from_str, ser::to_string};

use crate::{AB_LOCATOR, generated::{msnab_datatypes::types::{ContactType, ContactTypeEnum}, msnab_sharingservice::{bindings::{AbfindContactsPagedMessageSoapEnvelope, AbfindContactsPagedResponseMessageSoapEnvelope, AbgroupAddMessageSoapEnvelope}, factories::{ABGroupAddResponseFactory, ContactFactory, FindContactsPagedResponseFactory, UpdateDynamicItemResponseFactory}}}, MATRIX_CLIENT_LOCATOR, models::{msn_user::MSNUser, uuid::UUID}, MSN_CLIENT_LOCATOR, repositories::repository::Repository, web::error::WebError};

use super::webserver::DEFAULT_CACHE_KEY;

/* Address Book */
#[post("/abservice/abservice.asmx")]
pub async fn soap_adress_book_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    if let Some(soap_action_header) = request
        .headers()
        .get(HeaderName::from_str("SOAPAction").unwrap())
    {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()) {
            let name = soap_action.split("/").last().unwrap_or(soap_action);
            info!("{}Request: {}", &name, from_utf8(&body)?);

            match soap_action {
                "http://www.msn.com/webservices/AddressBook/ABFindContactsPaged" => {
                    return ab_find_contacts_paged(body, request).await;
                },
                "http://www.msn.com/webservices/AddressBook/ABGroupAdd" => {
                    return ab_group_add(body, request).await;
                },
                "http://www.msn.com/webservices/AddressBook/UpdateDynamicItem" => {
                    return update_dynamic_item(body, request).await;
                }

                _ => {}
            }
        } else {
            info!("AbService UnknownRequest: {}", from_utf8(&body)?);

        }
    }

    return Ok(HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
        .append_header(("Content-Type", "application/soap+xml"))
        .finish());
}




async fn ab_group_add(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;

    let request = from_str::<AbgroupAddMessageSoapEnvelope>(body)?;
        let new_group_guid = UUID::new(); //TODO change this when we really create the matrix space.
        let response = ABGroupAddResponseFactory::get_favorite_group_added_response(new_group_guid.to_string(), request.header.ok_or(Err(StatusCode::BAD_REQUEST))?.ab_auth_header.ticket_token);
        let response_serialized = to_string(&response)?;
    
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}


async fn ab_find_contacts_paged(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;

    let request = from_str::<AbfindContactsPagedMessageSoapEnvelope>(body)?;
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let ticket_token = &header.ab_auth_header.ticket_token;
    let matrix_token = header
        .ab_auth_header
        .ticket_token
        .substring(2, ticket_token.len())
        .to_string();

    let cache_key = &header.application_header.cache_key.unwrap_or_default();

    let msn_client = MSN_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let response : AbfindContactsPagedResponseMessageSoapEnvelope;
    
    let matrix_client =  MATRIX_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let me_mtx_id = msn_client.get_user().get_matrix_id();
    let me_display_name = matrix_client.account().get_display_name().await?.unwrap_or(msn_client.get_user_msn_addr());

    if header.application_header.partner_scenario.as_str() == "Initial" {
            //Fetch contacts from the ADL command
            let contacts_as_msn_usr = msn_client.get_contacts(false).await;
            let contact_list = msn_user_to_contact_type(&contacts_as_msn_usr);
            response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&me_mtx_id.to_string()),cache_key.clone(), msn_client.get_user_msn_addr(), me_display_name, contact_list);
        //    let empty_response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&me_mtx_id.to_string()),cache_key.clone(), msn_client.get_user_msn_addr(), me_display_name.clone(), Vec::new());
         //   response = empty_response;
        
    } else {
        let contact_list = AB_LOCATOR.get_contacts(&matrix_token).await.unwrap();
        response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&me_mtx_id.to_string()),cache_key.clone(), msn_client.get_user_msn_addr(), me_display_name, contact_list);
    }

    let response_serialized = to_string(&response)?;
    info!("find_contacts_paged_response: {}", response_serialized);
       
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

fn msn_user_to_contact_type(contacts: &Vec<MSNUser>) -> Vec<ContactType> {
    let mut out = Vec::new();
    for contact in contacts {
        let current_contact = ContactFactory::get_contact(&contact.get_uuid(), &contact.get_msn_addr(), &contact.get_display_name(), ContactTypeEnum::Live, false);
        out.push(current_contact);
    }
    return out;
}

async fn update_dynamic_item(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let response = UpdateDynamicItemResponseFactory::get_response(DEFAULT_CACHE_KEY.to_string());
    let response_serialized = to_string(&response)?;
    return Ok(HttpResponseBuilder::new(StatusCode::OK)
    .append_header(("Content-Type", "application/soap+xml"))
    .body(response_serialized));
}
