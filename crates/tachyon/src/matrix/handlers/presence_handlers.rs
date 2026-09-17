use matrix_sdk::event_handler::Ctx;
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::events::presence::PresenceEvent;
use crate::matrix::handlers::context::TachyonContext;

pub async fn handle_presence_event(
    event: PresenceEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {

    //TODO Handle presence Update

}


//TODO fallback when presence is disabled