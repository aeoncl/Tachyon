use crate::matrix::handlers::context::TachyonContext;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::message::{MessageEvent, MessageEventContent};
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::events::OriginalMessageLikeEvent;

pub async fn handle_message(
    event: MessageEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {


}