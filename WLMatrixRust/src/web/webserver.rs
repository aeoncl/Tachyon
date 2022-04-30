use std::io::Cursor;
use std::str::FromStr;
use std::sync::Arc;

use actix_web::{get, post, web, HttpRequest, HttpResponse, HttpResponseBuilder, Responder, Error};
use http::header::HeaderName;
use log::info;
use matrix_sdk::config::SyncSettings;
use matrix_sdk::media::{MediaFormat, MediaThumbnailSize, MediaRequest};
use matrix_sdk::ruma::api::client::media::get_content_thumbnail::v3::Method;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::{Client};
use mime::Mime;
use regex::Regex;

use std::str::from_utf8;
use substring::Substring;
use js_int::UInt;
use urlencoding::decode;

use http::StatusCode;
use lazy_static::lazy_static;
use matrix_sdk::ruma::{UserId, user_id, mxc_uri, MxcUri, OwnedUserId};
use reqwest::Url;
use yaserde::de::from_str;
use yaserde::ser::to_string;

use crate::generated::msnab_sharingservice::bindings::{
    AbfindContactsPagedMessageSoapEnvelope, FindMembershipMessageSoapEnvelope, AbgroupAddMessageSoapEnvelope,
};
use crate::generated::msnab_sharingservice::factories::{
    FindContactsPagedResponseFactory, FindMembershipResponseFactory, ABGroupAddResponseFactory, UpdateDynamicItemResponseFactory,
};
use crate::generated::msnstorage_service::bindings::{GetProfileMessageSoapEnvelope, UpdateDocumentMessageSoapEnvelope, UpdateDocumentResponseMessageSoapEnvelope, UpdateProfileMessageSoapEnvelope, DeleteRelationshipsMessageSoapEnvelope};
use crate::generated::msnstorage_service::factories::{GetProfileResponseFactory, UpdateDocumentResponseFactory, UpdateProfileResponseFactory, DeleteRelationshipsResponseFactory};
use crate::generated::msnstorage_service::types::StorageUserHeader;
use crate::generated::ppcrl_webservice::factories::RST2ResponseFactory;
use crate::generated::ppcrl_webservice::*;
use crate::models::uuid::UUID;
use crate::repositories::client_data_repository::{ClientDataRepository};
use crate::repositories::matrix_client_repository::MatrixClientRepository;
use crate::repositories::repository::Repository;
use crate::utils::identifiers::{msn_addr_to_matrix_id, get_matrix_device_id, get_hostname, parse_mxc};
use crate::{CLIENT_DATA_REPO, MATRIX_CLIENT_REPO};

use super::error::WebError;

lazy_static! {
    static ref DEFAULT_CACHE_KEY: String = String::from("12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA");
    static ref SHA1_REGEX: Regex = Regex::new(r"ru=([^&]*)&").unwrap();

}

lazy_static_include_bytes! {
    MSGR_CONFIG_XML => "assets/web/MsgrConfig.xml"
}

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[post("/RST2.srf")]
async fn rst2(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let test = std::str::from_utf8(&body).unwrap();

    let request_parsed: RST2RequestMessageSoapEnvelope = from_str(test).unwrap();
    let username_token = request_parsed.header.security.username_token.unwrap();

    let matrix_id = msn_addr_to_matrix_id(&username_token.username);
    let matrix_id_str = matrix_id.as_str();
    
    let matrix_user : OwnedUserId = <&UserId>::try_from(matrix_id_str).unwrap().to_owned();

    let url = Url::parse(format!("https://{}", matrix_user.server_name()).as_str())?;
    let client = Client::new(url).await?;

    let result = client
        .login(
            matrix_user.localpart(),
            username_token.password.as_str(),
            Some(get_matrix_device_id().as_str()),
            Some(get_hostname().as_str()),
        )
        .await?;
    
    let response = RST2ResponseFactory::get_rst2_success_response(
        result.access_token,
        username_token.username,
        UUID::from_string(&matrix_id),
    );

    let response_serialized = to_string(&response).unwrap();
    info!("RST2 Response: {}", &response_serialized);
    return Ok(HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "application/soap+xml"))
        .body(response_serialized));
}

#[get("/Config/MsgrConfig.asmx")]
async fn get_msgr_config() -> HttpResponse {
    let data: &'static [u8] = *MSGR_CONFIG_XML;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "application/soap+xml"))
        .body(data);
}

/* Address Book */
#[post("/abservice/abservice.asmx")]
async fn soap_adress_book_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
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

    let response = FindContactsPagedResponseFactory::get_empty_response(UUID::from_string(&msn_addr_to_matrix_id(&found.msn_login)),cache_key.clone(),found.msn_login.clone());

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

#[post("/abservice/SharingService.asmx")]
async fn soap_sharing_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    if let Some(soap_action_header) = request
        .headers()
        .get(HeaderName::from_str("SOAPAction").unwrap())
    {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()) {
            match soap_action {
                "http://www.msn.com/webservices/AddressBook/FindMembership" => {
                    return ab_sharing_find_membership(body, request).await;
                },
                _ => {}
            }
        }
    }
    return Ok(HttpResponseBuilder::new(StatusCode::NOT_FOUND)
        .append_header(("Content-Type", "application/soap+xml"))
        .finish());
}

async fn ab_sharing_find_membership(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body).unwrap();

    let request = from_str::<FindMembershipMessageSoapEnvelope>(body)?;
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let ticket_token = &header.ab_auth_header.ticket_token;
    let matrix_token = header.ab_auth_header.ticket_token.substring(2, ticket_token.len()).to_string();

    let cache_key = &header.application_header.cache_key .unwrap_or(DEFAULT_CACHE_KEY.to_string());

    let client_data_repo: Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();

    let found = client_data_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let response = FindMembershipResponseFactory::get_empty_response(
        UUID::from_string(&msn_addr_to_matrix_id(&found.msn_login)),
        found.msn_login.clone(),
        cache_key.clone());

    let response_serialized = to_string(&response)?;
    info!("find_membership_response: {}", response_serialized);
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

#[post("/storageservice/SchematizedStore.asmx")]
async fn soap_storage_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    if let Some(soap_action_header) = request
        .headers()
        .get(HeaderName::from_str("SOAPAction").unwrap())
    {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()) {
            match soap_action {
                "http://www.msn.com/webservices/storage/2008/GetProfile" => {
                    return storage_get_profile(body, request).await;
                },
                "http://www.msn.com/webservices/storage/2008/UpdateProfile" => {
                    return storage_update_profile(body, request).await;
                },
                "http://www.msn.com/webservices/storage/2008/UpdateDocument" => {
                    return storage_update_document(body, request).await;
                },
                "http://www.msn.com/webservices/storage/2008/ShareItem" => {
                    return share_item(body, request).await;
                },
                "http://www.msn.com/webservices/storage/2008/DeleteRelationships" => {
                    return delete_relationships(body, request).await;
                }

                _ => {}
            }
        }
    }
    return Ok(HttpResponseBuilder::new(StatusCode::NOT_FOUND)
        .append_header(("Content-Type", "application/soap+xml"))
        .finish());
}


#[derive(Deserialize)]
struct Info {
    mx_id: String,
    img_type: String
}

#[get("/storage/usertile/{mxc}/{img_type}")]
async fn get_profile_pic(path: web::Path<(String, String)>, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let (mxc, img_type) = path.into_inner();
    let mxc = from_utf8(&base64::decode(mxc.as_bytes())?)?.to_string();
    let mxc_as_str = mxc.as_str();
    let parsed_mxc = <&MxcUri>::try_from(mxc_as_str).unwrap().to_owned();

    
    let (server_part , mxc_part) = parse_mxc(&mxc);

    let homeserver_url = Url::parse(format!("https://{}", server_part).as_str())?;
    let client = Client::new(homeserver_url).await?;

    let media_request = MediaRequest{ source: MediaSource::Plain(parsed_mxc), format: MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap() })};
    let image = client.get_media_content(&media_request, false).await?;


   // let matrix_client = client_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
  // let documentstream = response.body.body.get_profile_response.get_profile_result.expression_profile.photo.document_streams.document_stream.get_mut(0).unwrap();

   //let profile_pic = matrix_client.account().get_avatar(MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap() })).await?.unwrap();
   //let encoded = base64::encode(&profile_pic);

        return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "image/jpeg")).body(image));
}

#[post("/ppsecure/sha1auth.srf")]
async fn sha1auth(body: web::Bytes) -> Result<HttpResponse, WebError> {
    let body = decode(from_utf8(&body)?)?.into_owned();
    let captures = SHA1_REGEX.captures(&body).unwrap();
    let redirect_url = decode(&captures[1])?.into_owned();
    info!("Redirect to {}", &redirect_url);
    return Ok(HttpResponseBuilder::new(StatusCode::FOUND).append_header(("Location", redirect_url.as_str())).finish());
}


async fn storage_get_profile(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;

    let request = from_str::<GetProfileMessageSoapEnvelope>(body)?;

    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let storage_user_header = header.storage_user_header.ok_or(StatusCode::BAD_REQUEST)?;
    let ticket_token = storage_user_header.ticket_token;
    let matrix_token = ticket_token.substring(2, ticket_token.len()).to_string();

    let client_data_repo: Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();

    let client = client_data_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
    let matrix_client = client_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let profile = matrix_client.account().get_profile().await?;
    let display_name = profile.displayname.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let psm = matrix_client.account().get_presence().await?.status_msg.unwrap_or_default();

    let mut img_mx_id : Option<String> = None;
    if let Some(avatar_url) = &matrix_client.account().get_avatar_url().await?{
        img_mx_id = Some(base64::encode(avatar_url.to_string()));
    }

    let response = GetProfileResponseFactory::get_response(UUID::from_string(&msn_addr_to_matrix_id(&client.msn_login)), DEFAULT_CACHE_KEY.to_string(), matrix_token, display_name, psm, img_mx_id);

    let response_serialized = to_string(&response)?;
    info!("get_profile_response: {}", response_serialized);
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

async fn storage_update_document(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;
    info!("update_document_request: {}", body);

    let request_deserialized = from_str::<UpdateDocumentMessageSoapEnvelope>(body).unwrap();

    let header = request_deserialized.header.ok_or(StatusCode::BAD_REQUEST)?;
    let storage_user_header = header.storage_user_header.ok_or(StatusCode::BAD_REQUEST)?;
    let matrix_token = extract_token_from_request(&storage_user_header);


    let client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
    let matrix_client = client_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let document_streams = request_deserialized.body.body.update_document.document.document_streams.document_stream;

    for document_stream in document_streams {
        if document_stream.document_stream_type == "UserTileStatic" {
            //We need to figure out the filetype from the content, because msn always sends png.
            let data_vector = base64::decode(document_stream.data.ok_or(StatusCode::BAD_REQUEST)?)?;
            let mime = get_mime_type(&data_vector);
            let mut data = Cursor::new(data_vector);
            let mtx_upload_response = matrix_client.upload(&mime, &mut data).await?;
            let mtx_avatar_response = matrix_client.account().set_avatar_url(Some(mtx_upload_response.content_uri.as_ref())).await?;
        }
    }

    let response = UpdateDocumentResponseFactory::get_response(matrix_token, header.affinity_cache_header.ok_or(StatusCode::BAD_REQUEST)?.cache_key.unwrap_or(DEFAULT_CACHE_KEY.to_string()));
    let response_serialized = to_string(&response)?;

    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));

}

fn get_mime_type(data_vector: &Vec<u8>) -> Mime {
    if &data_vector[0..3] == b"GIF" {
        return mime::IMAGE_GIF
    } else if &data_vector[0..2] == b"\xff\xd8" {
       return mime::IMAGE_JPEG
    } else if &data_vector[0..8] == b"\x89PNG\x0d\x0a\x1a\x0a"{
        return mime::IMAGE_PNG;
    }
    return mime::IMAGE_BMP;
}



async fn delete_relationships(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;
    info!("delete_relationships_request: {}", body);

    let request_deserialized : DeleteRelationshipsMessageSoapEnvelope = from_str(body)?;

    let header = request_deserialized.header.ok_or(StatusCode::BAD_REQUEST)?;
    let storage_user_header = header.storage_user_header.ok_or(StatusCode::BAD_REQUEST)?;
    let matrix_token = extract_token_from_request(&storage_user_header);


    if let Some(resource_id) = request_deserialized.body.body.delete_relationships.source_handle.resource_id {

        for object_handle in request_deserialized.body.body.delete_relationships.target_handles.object_handle {
            
            if let Some(current_res_id) = object_handle.resource_id {
                
                if(current_res_id.ends_with("205") && resource_id.ends_with("118")) {
                    //We are deleting a profile pic
                    let client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
                    let matrix_client = client_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
                    
                    let mtx_avatar_response = matrix_client.account().set_avatar_url(None).await?;
                }
            }
        }
    }

    let response = DeleteRelationshipsResponseFactory::get_response(matrix_token, DEFAULT_CACHE_KEY.to_string());
    let response_serialized = to_string(&response)?;

    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}


async fn storage_update_profile(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;
    info!("update_profile_request: {}", body);

    let body : UpdateProfileMessageSoapEnvelope = from_str(body)?;
    
    let header = body.header.ok_or(StatusCode::BAD_REQUEST)?;
    let storage_user_header = header.storage_user_header.ok_or(StatusCode::BAD_REQUEST)?;
    let matrix_token = extract_token_from_request(&storage_user_header);


    let profile = body.body.body.update_profile_request.profile.expression_profile;

    let client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
    let matrix_client = client_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(display_name) = profile.display_name {
        matrix_client.account().set_display_name(Some(display_name.as_str())).await?;
    }

    if let Some(psm) = profile.personal_status {
        let presence = matrix_client.account().get_presence().await?;
        matrix_client.account().set_presence(presence.presence, Some(psm.as_str())).await?;

    }

    let response = UpdateProfileResponseFactory::get_response(matrix_token, DEFAULT_CACHE_KEY.to_string());
    let response_serialized = to_string(&response)?;

    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

async fn share_item(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;
    info!("share_item_request: {}", body);
    let response = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\"><soap:Body><soap:Fault><faultcode>soap:Client</faultcode><faultstring>API ShareItem no longer supported</faultstring><faultactor>http://www.msn.com/webservices/AddressBook/ShareItem</faultactor><detail><errorcode xmlns=\"http://www.msn.com/webservices/AddressBook\">Forbidden</errorcode><errorstring xmlns=\"http://www.msn.com/webservices/AddressBook\">API ShareItem no longer supported</errorstring><machineName xmlns=\"http://www.msn.com/webservices/AddressBook\">DM2CDP1011931</machineName><additionalDetails><originalExceptionErrorMessage>API ShareItem no longer supported</originalExceptionErrorMessage></additionalDetails></detail></soap:Fault></soap:Body></soap:Envelope>");
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response));
}

fn extract_token_from_request(storage_header: &StorageUserHeader) -> String {
    let ticket_token = &storage_header.ticket_token;
    return ticket_token.substring(2, ticket_token.len()).to_string();
}