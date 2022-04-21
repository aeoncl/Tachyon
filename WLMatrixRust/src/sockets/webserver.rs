use std::io::Error;
use std::str::FromStr;
use std::sync::Arc;

use actix_web::http::header;
use actix_web::{get, web, Responder, HttpResponse, post, HttpRequest, HttpResponseBuilder};
use http::header::HeaderName;
use log::{trace, info};
use matrix_sdk::Client;
use substring::Substring;
use std::{str::from_utf8};

use matrix_sdk::ruma::UserId;
use reqwest::Url;
use yaserde::de::from_str;
use yaserde::ser::to_string;
use http::StatusCode;
use regex::Regex;
use lazy_static::lazy_static;

use crate::models::client_data::ClientData;
use crate::repositories::client_data_repository::{ClientDataRepository, Repository};
use crate::{MATRIX_CLIENT_REPO, CLIENT_DATA_REPO};
use crate::generated::msnab_sharingservice::bindings::{FindMembershipMessageSoapEnvelope, FindMembershipResponseMessageSoapEnvelope};
use crate::generated::msnab_sharingservice::factories::FindMembershipResponseFactory;
use crate::generated::ppcrl_webservice::*;
use crate::generated::ppcrl_webservice::factories::RST2ResponseFactory;
use crate::models::uuid::UUID;
use mime;

lazy_static_include_bytes! {
    MSGR_CONFIG_XML => "assets/web/MsgrConfig.xml"
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[post("/RST2.srf")]
async fn rst2(body: web::Bytes, request: HttpRequest) -> HttpResponse {
    let test = std::str::from_utf8(&body).unwrap();

    let request_parsed : RST2RequestMessageSoapEnvelope = from_str(test).unwrap();
    let username_token = request_parsed.header.security.username_token.unwrap();

    let matrix_id = msn_addr_to_matrix_id(&username_token.username);
    let matrix_user = UserId::try_from(matrix_id.clone()).unwrap();


    let client = Client::new(Url::parse(format!("https://{}", matrix_user.server_name()).as_str()).unwrap()).unwrap();

    let result = client.login(matrix_user.localpart(), username_token.password.as_str(), Some("WLMatrix"), None).await.unwrap(); //Todo handle login failure

    {
        MATRIX_CLIENT_REPO.lock().unwrap().insert(result.access_token.clone(), client);
        let client_date_repo : Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();
        client_date_repo.add(result.access_token.clone(), ClientData::new(username_token.username.to_string(), -1));

    }

    let response = RST2ResponseFactory::get_rst2_success_response(result.access_token, username_token.username, UUID::from_string(&matrix_id));

    let test = to_string(&response).unwrap();
    println!("DEBUG SOAP: {}", &test);
    return HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(test.clone());
}


#[get("/Config/MsgrConfig.asmx")]
async fn get_msgr_config() -> HttpResponse {
    let data: &'static [u8] = *MSGR_CONFIG_XML;
    return HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(data);
}

/* Address Book */
#[post("/abservice/abservice.asmx")]
async fn soap_adress_book_service(body: web::Bytes, request: HttpRequest) -> HttpResponse {
    if let Some(soap_action_header) = request.headers().get(HeaderName::from_str("SOAPAction").unwrap()) {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()){
            match soap_action {
                "todo" => {
                }

                _ =>  {
                }
            }
        }
        
    } 

    return HttpResponseBuilder::new(StatusCode::BAD_REQUEST).append_header(("Content-Type", "application/soap+xml")).finish();
}



#[post("/abservice/SharingService.asmx")]
async fn soap_sharing_service(body: web::Bytes, request: HttpRequest) -> HttpResponse {
    if let Some(soap_action_header) = request.headers().get(HeaderName::from_str("SOAPAction").unwrap()) {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()){
            match soap_action {
                "http://www.msn.com/webservices/AddressBook/FindMembership" => {
                    return ab_sharing_find_membership(body, request).await;
                }

                _ =>  {
                }
            }
        }
        
    } 
    return HttpResponseBuilder::new(StatusCode::NOT_FOUND).append_header(("Content-Type", "application/soap+xml")).finish();
}

async fn ab_sharing_find_membership(body: web::Bytes, request: HttpRequest) -> HttpResponse {
    let body = from_utf8(&body).unwrap();

    let mut out : HttpResponse = HttpResponseBuilder::new(StatusCode::BAD_REQUEST).append_header(("Content-Type", "application/soap+xml")).finish();

    if let Ok(request) = from_str::<FindMembershipMessageSoapEnvelope>(body){
        if let Some(header) = request.header {
            let ticket_token = &header.ab_auth_header.ticket_token;
            let matrix_token = header.ab_auth_header.ticket_token.substring(2, ticket_token.len()).to_string();

            let cache_key = &header.application_header.cache_key.unwrap_or_default();

            let client_data_repo : Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();
            
            if let Some(found) = client_data_repo.find(&matrix_token) {

             let response = FindMembershipResponseFactory::get_empty_response(UUID::from_string(&msn_addr_to_matrix_id(&found.msn_login)), found.msn_login.clone(), cache_key.clone());
             
             if let Ok(response_serialized) = to_string(&response){
                out = HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized);
             }

            };
        }
    }
    return out;

}


#[post("/storageservice/SchematizedStore.asmx")]
async fn soap_storage_service(body: web::Bytes, request: HttpRequest) -> HttpResponse {

    return HttpResponseBuilder::new(StatusCode::NOT_FOUND).append_header(("Content-Type", "application/soap+xml")).finish();

}

fn msn_addr_to_matrix_id(msn_addr: &String) -> String {

    lazy_static! {
        static ref MSN_ADDRESS_REGEX: Regex = Regex::new(r"(.+)@(.+)").unwrap();
    }

    let captures = MSN_ADDRESS_REGEX.captures(&msn_addr).unwrap();

    return format!("@{}:{}", captures[1].to_string(), captures[2].to_string()).to_string();
}

