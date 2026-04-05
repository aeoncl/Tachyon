use axum::http::StatusCode;
use axum::response::Response;
use matrix_sdk::Client;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::rsi::delete_messages::request::DeleteMessagesSoapEnvelope;
use msnp::soap::rsi::delete_messages::response::DeleteMessagesResponseSoapEnvelope;
use msnp::soap::traits::xml::ToXml;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::shared;

pub async fn delete_messages(request : DeleteMessagesSoapEnvelope, _token: TicketToken, _client: Client, client_data: &mut TachyonClient) -> Result<Response, RSIError> {

    let message_ids = request.body.body.message_ids.message_id;

    for message_id in message_ids {
        client_data.soap_holder().oims.remove(&message_id);
    }

    let soap_body = DeleteMessagesResponseSoapEnvelope::new();

    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}
