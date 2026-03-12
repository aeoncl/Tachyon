use crate::matrix::handlers;
use crate::matrix::handlers::context::TachyonContext;
use crate::tachyon::tachyon_client::TachyonClient;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;

pub(super) mod contact_handlers;
pub(super) mod context;
pub(super) mod membership_handlers;
pub(super) mod profile_handlers;
pub(super) mod presence_handlers;
mod message_handlers;

pub(super) fn register_event_handlers(matrix_client: &Client, client_data: TachyonClient) {

    matrix_client.add_event_handler_context(TachyonContext {
        client_data,
    });

    matrix_client.add_event_handler(
        |event: SyncRoomMemberEvent,
         room: Room,
         client: Client,
         context: Ctx<TachyonContext>| async move {
            println!("SyncRoomMemberEvent received: {:?}", &event);
            handlers::contact_handlers::handle_contacts(event, room, context, client).await;
        },
    );

    matrix_client.add_event_handler(
        |event: SyncRoomMemberEvent,
         room: Room,
         client: Client,
         context: Ctx<TachyonContext>| async move {
            println!("SyncRoomMemberEvent received: {:?}", &event);
            handlers::membership_handlers::handle_memberships(event, room, context, client).await;
        },
    );

    matrix_client.add_event_handler(
        |event: StrippedRoomMemberEvent,
         room: Room,
         client: Client,
         context: Ctx<TachyonContext>| async move {
            println!("StrippedRoomMemberEvent received: {:?}", &event);
            handlers::contact_handlers::handle_contacts_stripped(event, room, context, client).await;
        },
    );

    matrix_client.add_event_handler(
        |event: StrippedRoomMemberEvent,
         room: Room,
         client: Client,
         context: Ctx<TachyonContext>| async move {
            println!("StrippedRoomMemberEvent received: {:?}", &event);
            handlers::membership_handlers::handle_memberships_stripped(
                event, room, context, client,
            ).await;
        },
    );

    matrix_client.add_event_handler(
        |event: OriginalSyncRoomMessageEvent,
         room: Room,
         client: Client,
         context: Ctx<TachyonContext>| async move {
            println!("OriginalSyncRoomMessageEvent received: {:?}", &event);
            handlers::message_handlers::handle_message(
                event, room, context, client,
            ).await;
        },
    );

}