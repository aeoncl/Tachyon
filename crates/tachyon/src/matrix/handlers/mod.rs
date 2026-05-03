use log::debug;
use crate::matrix::handlers;
use crate::matrix::handlers::context::TachyonContext;
use crate::tachyon::client::tachyon_client::TachyonClient;
use matrix_sdk::event_handler::{Ctx, EventHandler, EventHandlerDropGuard, EventHandlerHandle};
use matrix_sdk::ruma::events::room::member::{StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::{Client, Room};
use matrix_sdk::ruma::events::key::verification::request::ToDeviceKeyVerificationRequestEvent;
use matrix_sdk::ruma::events::room::message::OriginalSyncRoomMessageEvent;
use matrix_sdk::ruma::events::room::tombstone::{OriginalSyncRoomTombstoneEvent, RoomTombstoneEvent, SyncRoomTombstoneEvent};
use matrix_sdk::ruma::events::typing::SyncTypingEvent;
use crate::matrix::handlers::request_verification_handlers::request_verification_handler;

pub mod contact_handlers;
pub(super) mod context;
pub mod membership_handlers;
pub(super) mod profile_handlers;
pub(super) mod presence_handlers;
mod message_handlers;
mod request_verification_handlers;

pub struct EventHandlerContextDropGuard<C: Clone + Send + Sync + 'static> {
    matrix_client: Client,
    _phantom: std::marker::PhantomData<C>
}

impl<C: Clone + Send + Sync + 'static> EventHandlerContextDropGuard<C> {
    pub fn new(matrix_client: Client) -> Self {
        Self {
            matrix_client,
            _phantom: Default::default(),
        }
    }
}

impl<C: Clone + Send + Sync + 'static> Drop for EventHandlerContextDropGuard<C> {
    fn drop(&mut self) {
        self.matrix_client.add_event_handler_context(None::<C>);
    }
}

fn register_droppable_event_handler(matrix_client: &Client, event_drop_guards: &mut Vec<EventHandlerDropGuard>, create_handler: impl FnOnce() -> EventHandlerHandle) {
    let handler = create_handler();
    let drop_guard = matrix_client.event_handler_drop_guard(handler);
    event_drop_guards.push(drop_guard);
}

fn register_droppable_event_handler_context<C: Clone + Send + Sync + 'static>(matrix_client: &Client, context: C) -> EventHandlerContextDropGuard<C> {
    matrix_client.add_event_handler_context(Some(context));
    EventHandlerContextDropGuard::new(matrix_client.clone())
}

pub(super) fn register_event_handlers(matrix_client: &Client, tachyon_client: TachyonClient) -> (Vec<EventHandlerDropGuard>, EventHandlerContextDropGuard<TachyonContext>) {

    let mut event_drop_guards = Vec::new();
    let context_drop_guard = register_droppable_event_handler_context(matrix_client, TachyonContext { tachyon_client });

    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {
        matrix_client.add_event_handler(
            |event: SyncRoomMemberEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {
                debug!("SyncRoomMemberEvent received: {:?}", &event);

                let context = context.as_ref().unwrap().clone();

                handlers::contact_handlers::handle_contacts(event, room, context.tachyon_client, client).await;
            },
        )
    });


    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {
        matrix_client.add_event_handler(
            |event: SyncRoomMemberEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {
                debug!("SyncRoomMemberEvent received: {:?}", &event);
                let context = context.as_ref().unwrap().clone();
                handlers::membership_handlers::handle_memberships(event, room, context.tachyon_client, client).await;
            },
        )
    });

    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {
        matrix_client.add_event_handler(
            |event: StrippedRoomMemberEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {
                debug!("StrippedRoomMemberEvent received: {:?}", &event);
                let context = context.as_ref().unwrap().clone();
                handlers::contact_handlers::handle_contacts_stripped(event, room, context.tachyon_client, client).await;
            },
        )
    });

    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {

        matrix_client.add_event_handler(
            |event: StrippedRoomMemberEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {
                debug!("StrippedRoomMemberEvent received: {:?}", &event);

                let context = context.as_ref().unwrap().clone();


                handlers::membership_handlers::handle_memberships_stripped(
                    event, room, context.tachyon_client, client,
                ).await;
            },
        )

    });

    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {

        matrix_client.add_event_handler(
            |event: OriginalSyncRoomTombstoneEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {

                let context = context.as_ref().unwrap().clone();

                debug!("StrippedRoomMemberEvent received: {:?}", &event);
                handlers::contact_handlers::handle_tombstone(
                    event, room, context.tachyon_client, client,
                ).await;
            },
        )

    });

    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {

        matrix_client.add_event_handler(
            |event: OriginalSyncRoomTombstoneEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {

                let context = context.as_ref().unwrap().clone();


                debug!("StrippedRoomMemberEvent received: {:?}", &event);
                handlers::membership_handlers::handle_tombstone(
                    event, room, context.tachyon_client, client,
                ).await;
            },
        )

    });


    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {

        matrix_client.add_event_handler(
            |event: OriginalSyncRoomMessageEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {

                let context = context.as_ref().unwrap().clone();


                debug!("OriginalSyncRoomMessageEvent received: {:?}", &event);
                handlers::message_handlers::handle_message(
                    event, room, context.tachyon_client, client,
                ).await;
            },
        )

    });


    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {
        matrix_client.add_event_handler(
            |event: SyncTypingEvent,
             room: Room,
             client: Client,
             context: Ctx<Option<TachyonContext>>| async move {
                debug!("OriginalSyncTypingEvent received: {:?}", &event);

                let context = context.as_ref().unwrap().clone();

                handlers::message_handlers::handle_typing_notice(
                    event, room, context.tachyon_client, client,
                ).await;
            },
        )
    });

    register_droppable_event_handler(matrix_client, &mut event_drop_guards, || {
        matrix_client.add_event_handler(
            |ev: ToDeviceKeyVerificationRequestEvent, client: Client| async move {
                let request = client
                    .encryption()
                    .get_verification_request(&ev.sender, &ev.content.transaction_id)
                    .await
                    .expect("Request object wasn't created");

                tokio::spawn(request_verification_handler(client, request));
            },
        )
    });

    (event_drop_guards, context_drop_guard)
}