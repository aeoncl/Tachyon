use matrix_sdk::{
    event_handler::Ctx,
    ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent},
    Client, Room,
};

use crate::matrix::handlers::context::TachyonContext;

pub async fn handle_memberships(
    event: SyncRoomMemberEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {
}

pub async fn handle_memberships_stripped(
    event: StrippedRoomMemberEvent,
    room: Room,
    context: Ctx<TachyonContext>,
    client: Client,
) {
}
