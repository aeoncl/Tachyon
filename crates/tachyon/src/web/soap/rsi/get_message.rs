use anyhow::anyhow;
use axum::response::Response;
use matrix_sdk::Client;
use matrix_sdk::room::MessagesOptions;
use matrix_sdk::ruma::{EventId, RoomId};
use matrix_sdk::ruma::events::{AnyMessageLikeEvent, AnySyncMessageLikeEvent, AnySyncTimelineEvent, AnyTimelineEvent, MessageLikeEvent, SyncMessageLikeEvent};
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::sharing_service::find_membership::request::FindMembershipRequestSoapEnvelope;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use crate::notification::client_store::ClientStoreFacade;
use crate::web::soap::error::ABError;
use crate::web::soap::rsi::error::RSIError;

pub async fn get_message(request : GetMessageMessageSoapEnvelope, token: TicketToken, client: Client, client_store: &ClientStoreFacade) -> Result<Response, RSIError> {

    let message_id = request.body.body.message_id;
    todo!()



}