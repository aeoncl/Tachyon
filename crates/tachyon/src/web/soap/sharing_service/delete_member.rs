use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use matrix_sdk::Client;

use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::sharing_service::delete_member::request::DeleteMemberMessageSoapEnvelope;
use msnp::soap::abch::sharing_service::delete_member::response::DeleteMemberResponseMessageSoapEnvelope;
use msnp::soap::traits::xml::ToXml;

use crate::notification::client_store::ClientData;
use crate::web::soap::error::ABError;
use crate::web::soap::shared;

pub async fn delete_member(request : DeleteMemberMessageSoapEnvelope, token: TicketToken, client: Client, mut client_data: ClientData) -> Result<Response, ABError> {
    let cache_key = &request.header.ok_or(anyhow!("Header missing"))?.application_header.cache_key.unwrap_or_default();

    let soap_body = DeleteMemberResponseMessageSoapEnvelope::new(cache_key);

    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}