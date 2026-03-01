use crate::matrix::handlers::context::TachyonContext;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::direct::DirectEvent;
use matrix_sdk::{Client, Room, RoomState};
use ruma::events::room::create::RoomCreateEvent;

pub async fn event_handler(
    event: RoomCreateEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    _client: Client,
) {

    match room.state() {
        RoomState::Joined => {

        }
        RoomState::Left => {

        }
        RoomState::Invited => {

        }
        RoomState::Knocked => {

        }
        RoomState::Banned => {

        }
    }


}
