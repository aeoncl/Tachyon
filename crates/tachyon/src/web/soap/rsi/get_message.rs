use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use matrix_sdk::Client;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use msnp::soap::rsi::get_message::response::GetMessageResponseMessageSoapEnvelope;
use msnp::soap::traits::xml::ToXml;
use crate::notification::client_store::ClientData;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::shared;

pub async fn get_message(request : GetMessageMessageSoapEnvelope, _token: TicketToken, _client: Client, client_data: &mut ClientData) -> Result<Response, RSIError> {

    let message_id = request.body.body.message_id;
    let mark_as_read = request.body.body.also_mark_as_read;

    if mark_as_read {
        match client_data.get_oims().get_mut(&message_id) {
            None => {
                Err(RSIError::InternalServerError(anyhow!("No Content")))
            }
            Some(mut oim_content) => {
                let soap_body = GetMessageResponseMessageSoapEnvelope::new(oim_content.clone());
                oim_content.read = true;
                Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
            }
        }
    } else {
        match client_data.get_oims().get(&message_id) {
            None => {
                Err(RSIError::InternalServerError(anyhow!("No Content")))
            }
            Some(oim_content) => {
                let soap_body = GetMessageResponseMessageSoapEnvelope::new(oim_content.clone());
                Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
            }
        }
    }





}
