use axum::http::StatusCode;
use axum::response::Response;
use matrix_sdk::Client;

use msnp::shared::models::oim::{MetaData, MetadataMessage};
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::rsi::get_metadata::request::GetMetadataMessageSoapEnvelope;
use msnp::soap::rsi::get_metadata::response::GetMetadataResponseMessageSoapEnvelope;
use msnp::soap::traits::xml::ToXml;

use crate::notification::client_store::ClientData;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::shared;

pub async fn get_metadata(request : GetMetadataMessageSoapEnvelope, token: TicketToken, client: Client, client_data: &mut ClientData) -> Result<Response, RSIError> {

    let mut md = MetaData {
        ..Default::default()
    };

    for oim in client_data.get_oims().iter() {
        let oim = oim.value();
        let serialized = oim.to_string();

        let display_name = oim.sender_display_name.as_ref().unwrap_or(&oim.sender.to_string()).to_owned();
        let metadata_message = MetadataMessage::new(oim.recv_datetime.clone(), oim.sender.clone(), display_name, oim.message_id.clone(), serialized.len(), oim.read);
        md.messages.push(metadata_message);
    }

    let soap_body = GetMetadataResponseMessageSoapEnvelope::new(md);
    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}