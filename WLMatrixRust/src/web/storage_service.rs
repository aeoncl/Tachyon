use std::{str::{from_utf8, FromStr}, sync::Arc};

use actix_web::{post, web, HttpRequest, HttpResponse, HttpResponseBuilder, get};
use base64::{Engine, engine::general_purpose};
use http::{header::HeaderName, StatusCode};
use js_int::UInt;
use log::info;
use matrix_sdk::{ruma::{MxcUri, events::room::MediaSource, api::client::media::get_content_thumbnail::v3::Method}, Client, media::{MediaRequest, MediaFormat, MediaThumbnailSize}};
use mime::Mime;
use substring::Substring;
use yaserde::{ser::to_string, de::from_str};
use crate::{web::{error::WebError, webserver::{DEFAULT_CACHE_KEY}}, generated::msnstorage_service::{bindings::{GetProfileMessageSoapEnvelope, DeleteRelationshipsMessageSoapEnvelope, UpdateProfileMessageSoapEnvelope, UpdateDocumentMessageSoapEnvelope}, factories::{GetProfileResponseFactory, UpdateDocumentResponseFactory, DeleteRelationshipsResponseFactory, UpdateProfileResponseFactory}, types::StorageUserHeader}, repositories::{repository::Repository}, models::uuid::UUID, utils::identifiers::{parse_mxc}, MSN_CLIENT_LOCATOR, MATRIX_CLIENT_LOCATOR};



#[post("/storageservice/SchematizedStore.asmx")]
pub async fn soap_storage_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
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
pub async fn get_profile_pic(path: web::Path<(String, String)>, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let (mxc, img_type) = path.into_inner();
    let mxc = from_utf8(&general_purpose::STANDARD.decode(mxc.as_bytes())?)?.to_string();
    let mxc_as_str = mxc.as_str();
    let parsed_mxc = <&MxcUri>::try_from(mxc_as_str).unwrap().to_owned();

    
    let (server_part , mxc_part) = parse_mxc(&mxc);

    // let homeserver_url = Url::parse(format!("https://{}", server_part).as_str())?;

    let client= Client::builder().disable_ssl_verification().server_name(parsed_mxc.server_name().unwrap()).build().await?;
    
    let media_request = MediaRequest{ source: MediaSource::Plain(parsed_mxc), format: MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap() })};
    let image = client.media().get_media_content(&media_request, false).await?;
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "image/jpeg")).body(image));

    


   // let matrix_client = client_repo.find(&matrix_token).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
  // let documentstream = response.body.body.get_profile_response.get_profile_result.expression_profile.photo.document_streams.document_stream.get_mut(0).unwrap();

   //let profile_pic = matrix_client.account().get_avatar(MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap() })).await?.unwrap();
   //let encoded = base64::encode(&profile_pic);

}

async fn storage_get_profile(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;

    let request = from_str::<GetProfileMessageSoapEnvelope>(body)?;

    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let storage_user_header = header.storage_user_header.ok_or(StatusCode::BAD_REQUEST)?;
    let ticket_token = storage_user_header.ticket_token;
    let matrix_token = ticket_token.substring(2, ticket_token.len()).to_string();


    let msn_client = MSN_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let me = msn_client.get_user();

    let matrix_client =  MATRIX_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let profile = matrix_client.account().get_profile().await?;
    let display_name = profile.displayname.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let psm = matrix_client.account().get_presence().await?.status_msg.unwrap_or_default();

    let mut img_mx_id : Option<String> = None;
    if let Some(avatar_url) = &matrix_client.account().get_avatar_url().await?{
        img_mx_id = Some(general_purpose::STANDARD.encode(avatar_url.to_string()));
    }



    let response = GetProfileResponseFactory::get_response(me.get_uuid(), DEFAULT_CACHE_KEY.to_string(), matrix_token, display_name, psm, img_mx_id);

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


    let matrix_client = MATRIX_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let document_streams = request_deserialized.body.body.update_document.document.document_streams.document_stream;

    for document_stream in document_streams {
        if document_stream.document_stream_type == "UserTileStatic" {
            //We need to figure out the filetype from the content, because msn always sends png.
            let data_vector = general_purpose::STANDARD.decode(document_stream.data.ok_or(StatusCode::BAD_REQUEST)?)?;
            let mime = get_mime_type(&data_vector);

            let mtx_upload_response = matrix_client.account().upload_avatar(&mime, data_vector).await?;

            let mtx_avatar_response = matrix_client.account().set_avatar_url(Some(mtx_upload_response.as_ref())).await?;
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
                    let matrix_client = MATRIX_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
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

    let matrix_client = MATRIX_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(display_name) = profile.display_name {
        matrix_client.account().set_display_name(Some(display_name.as_str())).await?;
    }

    let psm = profile.personal_status.unwrap_or(String::new());
    let presence = matrix_client.account().get_presence().await?;
    matrix_client.account().set_presence(presence.presence, Some(psm)).await?;

    

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