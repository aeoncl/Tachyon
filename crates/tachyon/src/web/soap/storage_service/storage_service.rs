use std::str::FromStr;
use anyhow::anyhow;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use base64::Engine;
use base64::engine::general_purpose;
use log::error;
use matrix_sdk::Client;
use mime::Mime;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::storage_service::delete_relationships::request::DeleteRelationshipsMessageSoapEnvelope;
use msnp::soap::storage_service::delete_relationships::response::DeleteRelationshipsResponseMessageSoapEnvelope;
use msnp::soap::storage_service::get_profile::request::GetProfileMessageSoapEnvelope;
use msnp::soap::storage_service::get_profile::response::GetProfileResponseMessageSoapEnvelope;
use msnp::soap::storage_service::headers::StorageServiceRequestSoapEnvelope;
use msnp::soap::storage_service::share_item::request::ShareItemMessageSoapEnvelope;
use msnp::soap::storage_service::upate_document::request::UpdateDocumentMessageSoapEnvelope;
use msnp::soap::storage_service::upate_document::response::UpdateDocumentResponseMessageSoapEnvelope;
use msnp::soap::storage_service::update_profile::request::UpdateProfileMessageSoapEnvelope;
use msnp::soap::storage_service::update_profile::response::UpdateProfileResponseMessageSoapEnvelope;
use msnp::soap::traits::xml::{ToXml, TryFromXml};
use crate::tachyon::client_store::ClientStoreFacade;
use crate::tachyon::identifiers::MatrixIdCompatible;
use crate::tachyon::traits::ToUuid;
use crate::web::soap::error::ABError;
use crate::web::soap::shared;
use crate::web::web_endpoints::DEFAULT_CACHE_KEY;
pub async fn storage_service(headers: HeaderMap, State(state): State<ClientStoreFacade>, body: String) -> Result<Response, ABError> {

    let soap_action = headers.get("SOAPAction").ok_or(ABError::MissingHeader("SOAPAction".into()))?.to_str()?.trim_start_matches("\"").trim_end_matches("\"");

    let header_env = StorageServiceRequestSoapEnvelope::try_from_xml(&body)?;
    let token = TicketToken::from_str(&header_env.header.storage_user.unwrap().ticket_token).unwrap();

    let client_data = state.get_client(&token.0).ok_or(ABError::AuthenticationFailed {source: anyhow!("Expected Client Data to be present in client Store")})?;

    let client = client_data.matrix_client();
    
    let client_token = client.access_token().ok_or(ABError::AuthenticationFailed {source: anyhow!("No Token present in Matrix Client")})?;
    if token != client_token {
        return Err(ABError::AuthenticationFailed { source: anyhow!("Supplied Token & Matrix Token don't match: {} == {}", &token.0, &client_token) });
    }

    match soap_action {
        "http://www.msn.com/webservices/storage/2008/GetProfile" => {
            get_profile(GetProfileMessageSoapEnvelope::try_from_xml(&body)?, token, client).await
        },
        "http://www.msn.com/webservices/storage/2008/UpdateProfile" => {
            update_profile(UpdateProfileMessageSoapEnvelope::try_from_xml(&body)?, token, client).await
        },
        "http://www.msn.com/webservices/storage/2008/UpdateDocument" => {
            update_document(UpdateDocumentMessageSoapEnvelope::try_from_xml(&body)?, token, client).await
        }
        "http://www.msn.com/webservices/storage/2008/DeleteRelationships" => {
            delete_relationships(DeleteRelationshipsMessageSoapEnvelope::try_from_xml(&body)?, token, client).await
        }
        "http://www.msn.com/webservices/storage/2008/ShareItem" => {
            share_item(ShareItemMessageSoapEnvelope::try_from_xml(&body)?, token, client).await
        }
        _ => {
            error!("SOAP|ABCH: Unsupported soap action: {}", &soap_action);
            Err(ABError::UnsupportedSoapAction(soap_action.to_string()))
        }
    }


}

async fn share_item(request: ShareItemMessageSoapEnvelope, _token: TicketToken, client: Client) -> Result<Response, ABError> {

    //TODO
    let response = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\"><soap:Body><soap:Fault><faultcode>soap:Client</faultcode><faultstring>API ShareItem no longer supported</faultstring><faultactor>http://www.msn.com/webservices/AddressBook/ShareItem</faultactor><detail><errorcode xmlns=\"http://www.msn.com/webservices/AddressBook\">Forbidden</errorcode><errorstring xmlns=\"http://www.msn.com/webservices/AddressBook\">API ShareItem no longer supported</errorstring><machineName xmlns=\"http://www.msn.com/webservices/AddressBook\">DM2CDP1011931</machineName><additionalDetails><originalExceptionErrorMessage>API ShareItem no longer supported</originalExceptionErrorMessage></additionalDetails></detail></soap:Fault></soap:Body></soap:Envelope>");
    Ok(shared::build_soap_response(response, StatusCode::OK))

}

async fn delete_relationships(request: DeleteRelationshipsMessageSoapEnvelope, _token: TicketToken, client: Client) -> Result<Response, ABError> {

    if let Some(resource_id) = request.body.body.source_handle.resource_id {

        for object_handle in request.body.body.target_handles.object_handle {

            if let Some(current_res_id) = object_handle.resource_id {

                if current_res_id.ends_with("205") && resource_id.ends_with("118") {
                    //We are deleting a profile pic
                    let mtx_avatar_response = client.account().set_avatar_url(None).await?;
                }
            }
        }
    }

    let soap_body = DeleteRelationshipsResponseMessageSoapEnvelope::new(DEFAULT_CACHE_KEY.to_string());
    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

}

async fn update_document(request: UpdateDocumentMessageSoapEnvelope, _token: TicketToken, client: Client) -> Result<Response, ABError> {
    let document_streams = request.body.body.document.document_streams.document_stream;

    for document_stream in document_streams {
        if document_stream.document_stream_type == "UserTileStatic" {
            //We need to figure out the filetype from the content, because msn always sends png.
            let data_vector = general_purpose::STANDARD.decode(document_stream.data.ok_or(anyhow!("Document stream contained no data"))
                ?)
                .map_err(|e| anyhow!("Failed to decode base64 document stream data: {}", e))?;

            let mime = get_mime_type(&data_vector);

            let mtx_upload_response = client.account().upload_avatar(&mime, data_vector).await?;

            let mtx_avatar_response = client.account().set_avatar_url(Some(mtx_upload_response.as_ref())).await?;
        }
    }

    let soap_body = UpdateDocumentResponseMessageSoapEnvelope::new(DEFAULT_CACHE_KEY.to_string());
    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

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


async fn get_profile(_request: GetProfileMessageSoapEnvelope, _token: TicketToken, matrix_client: Client) -> Result<Response, ABError> {
    let user_id = matrix_client.user_id().ok_or(anyhow!("Expected to have user_id in matrix client"))?;
    let msn_addr = EmailAddress::from_user_id(user_id);
    let uuid = msn_addr.to_uuid();

    let display_name = matrix_client.account().get_display_name().await?.unwrap_or(msn_addr.to_string());

    let avatar_mxid = matrix_client.account().get_avatar_url().await?.map(|a| general_purpose::STANDARD.encode(a.as_str()));

    let soap_body = GetProfileResponseMessageSoapEnvelope::new(uuid, DEFAULT_CACHE_KEY.to_string(), display_name, String::new(), avatar_mxid);
    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

}

async fn update_profile(request: UpdateProfileMessageSoapEnvelope, _token: TicketToken, matrix_client: Client) -> Result<Response, ABError> {
    let profile = request.body.body.profile.expression_profile;

    if let Some(display_name) = profile.display_name {
        matrix_client.account().set_display_name(Some(display_name.as_str())).await?;
    }

    let psm = profile.personal_status.unwrap_or(String::new());

    let soap_body = UpdateProfileResponseMessageSoapEnvelope::new(DEFAULT_CACHE_KEY.to_string());
    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

}

