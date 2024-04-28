use anyhow::anyhow;
use axum::response::Response;
use matrix_sdk::Client;
use matrix_sdk::room::MessagesOptions;
use matrix_sdk::ruma::{EventId, RoomId};
use matrix_sdk::ruma::events::{AnyMessageLikeEvent, AnySyncMessageLikeEvent, AnySyncTimelineEvent, AnyTimelineEvent, MessageLikeEvent, SyncMessageLikeEvent};
use matrix_sdk::ruma::events::room::message::MessageType;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::sharing_service::find_membership::request::FindMembershipRequestSoapEnvelope;
use msnp::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::web::soap::error::ABError;
use crate::web::soap::rsi::error::RSIError;

pub async fn get_message(request : GetMessageMessageSoapEnvelope, token: TicketToken, client: Client, client_data: &mut ClientData) -> Result<Response, RSIError> {

    let message_id = request.body.body.message_id;

    let mut oims = client_data.get_oims();


    for i in 0..oims.len(){
        let current = oims.get(i).expect("to be here");
        let evend_id = current.event_id().as_str();
        let room_id = "";
        //TODO room id
        let oim_id = format!("{};{}", &room_id, &evend_id);

        if oim_id == message_id {
            //ITS THIS ONE.


            match current {
                AnySyncMessageLikeEvent::RoomMessage(SyncMessageLikeEvent::Original(original_event)) => {
                    if let MessageType::Text(text) = &original_event.content.msgtype {


                    }
                }
                _ => {}
            }




            oims.remove(i);
            break;
        }



    }

    todo!()

}