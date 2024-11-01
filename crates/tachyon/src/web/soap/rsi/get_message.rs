use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use matrix_sdk::Client;
use matrix_sdk::room::MessagesOptions;
use matrix_sdk::ruma::{EventId, RoomId};
use matrix_sdk::ruma::events::{AnyMessageLikeEvent, AnySyncMessageLikeEvent, AnySyncTimelineEvent, AnyTimelineEvent, MessageLikeEvent, SyncMessageLikeEvent};
use matrix_sdk::ruma::events::room::message::MessageType;
use msnp::shared::models::oim::OIM;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::sharing_service::find_membership::request::FindMembershipRequestSoapEnvelope;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use msnp::soap::rsi::get_message::response::GetMessageResponseMessageSoapEnvelope;
use msnp::soap::rsi::get_metadata::response::GetMetadataResponseMessageSoapEnvelope;
use msnp::soap::traits::xml::ToXml;
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::web::soap::error::ABError;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::shared;

pub async fn get_message(request : GetMessageMessageSoapEnvelope, token: TicketToken, client: Client, client_data: &mut ClientData) -> Result<Response, RSIError> {

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