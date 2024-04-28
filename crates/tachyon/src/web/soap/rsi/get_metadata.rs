use axum::http::StatusCode;
use axum::response::Response;
use chrono::{DateTime, Local, TimeZone};
use matrix_sdk::Client;
use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, SyncMessageLikeEvent};
use matrix_sdk::ruma::events::room::message::MessageType;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::oim::{MetaData, MetadataMessage};
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use msnp::soap::rsi::get_metadata::request::GetMetadataMessageSoapEnvelope;
use msnp::soap::rsi::get_metadata::response::GetMetadataResponseMessageSoapEnvelope;
use msnp::soap::traits::xml::ToXml;
use crate::notification::client_store::{ClientData, ClientDataInner, ClientStoreFacade};
use crate::shared::identifiers::MatrixIdCompatible;
use crate::web::soap::rsi::error::RSIError;
use crate::web::soap::shared;

pub async fn get_message(request : GetMetadataMessageSoapEnvelope, token: TicketToken, client: Client, client_data: &mut ClientData) -> Result<Response, RSIError> {


    let mut md = MetaData {
        ..Default::default()
    };

    for oim in client_data.get_oims().iter() {
        match oim {
            AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Original(original_event)) => {


                if let MessageType::Text(text) = &original_event.content.msgtype {
                    let room_id = "";
                    //TODO add room id somewhere
                    println!("DEBUG: {}", text.body);
                    let timestamp = DateTime::from_timestamp_millis(original_event.origin_server_ts.0.into()).unwrap().naive_local();
                    let message = MetadataMessage::new(Local.from_local_datetime(&timestamp).unwrap(), EmailAddress::from_user_id(&original_event.sender), "blabla".into(), format!("{};{}", &room_id, &original_event.event_id), 0);
                    md.messages.push(message);
                }
            }
            _ => {

            }
        }
    }

    let soap_body = GetMetadataResponseMessageSoapEnvelope::new(md);
    Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
}