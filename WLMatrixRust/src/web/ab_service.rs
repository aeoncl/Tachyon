use std::{str::{from_utf8, FromStr}, sync::Arc};

use actix_web::{post, HttpRequest, HttpResponseBuilder, HttpResponse, web};
use http::{header::HeaderName, StatusCode};
use log::info;
use substring::Substring;
use yaserde::{ser::to_string, de::from_str};

use crate::{web::error::WebError, generated::msnab_sharingservice::{bindings::{AbgroupAddMessageSoapEnvelope, AbfindContactsPagedMessageSoapEnvelope, AbfindContactsPagedResponseMessageSoapEnvelope}, factories::{ABGroupAddResponseFactory, FindContactsPagedResponseFactory, UpdateDynamicItemResponseFactory}}, repositories::{client_data_repository::ClientDataRepository, repository::Repository}, CLIENT_DATA_REPO, utils::identifiers::msn_addr_to_matrix_id, models::uuid::UUID, AB_DATA_REPO};

use super::webserver::DEFAULT_CACHE_KEY;




/* Address Book */
#[post("/abservice/abservice.asmx")]
pub async fn soap_adress_book_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    if let Some(soap_action_header) = request
        .headers()
        .get(HeaderName::from_str("SOAPAction").unwrap())
    {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()) {
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

    let client_data_repo: Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();

    let found = client_data_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let response : AbfindContactsPagedResponseMessageSoapEnvelope;

    if header.application_header.partner_scenario.as_str() == "Initial" {
        response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&msn_addr_to_matrix_id(&found.msn_login)),cache_key.clone(),found.msn_login.clone(), Vec::new());
    } else {
        let ab_data_repo  = AB_DATA_REPO.clone();
        let mut ab_data = ab_data_repo.find_mut(&matrix_token).unwrap();
        let contact_list = ab_data.consume_contact_list();
        response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&msn_addr_to_matrix_id(&found.msn_login)),cache_key.clone(),found.msn_login.clone(), contact_list);
    }

    let response_serialized = to_string(&response)?;
    info!("find_contacts_paged_response: {}", response_serialized);
       
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

async fn update_dynamic_item(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let response = UpdateDynamicItemResponseFactory::get_response(DEFAULT_CACHE_KEY.to_string());
    let response_serialized = to_string(&response)?;
    return Ok(HttpResponseBuilder::new(StatusCode::OK)
    .append_header(("Content-Type", "application/soap+xml"))
    .body(response_serialized));
}