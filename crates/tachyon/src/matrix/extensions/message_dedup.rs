
use dashmap::DashMap;
use lazy_static::lazy_static;
use matrix_sdk::{Error, Room};
use matrix_sdk::room::futures::{SendMessageLikeEvent, SendMessageLikeEventResult};
use matrix_sdk::ruma::events::MessageLikeEventContent;
use matrix_sdk::ruma::{EventId, OwnedEventId, RoomId, OwnedRoomId};
use tokio::sync::Mutex;
use std::sync::Arc;

lazy_static! {
    static ref DEDUP_STATE: DashMap<OwnedRoomId, RoomDedupState> = DashMap::new();
}

#[derive(Clone)]
struct RoomDedupState {
    send_lock: Arc<Mutex<()>>,
    deduped_events: Arc<DashMap<OwnedEventId, ()>>,
}

impl RoomDedupState {
    fn new() -> Self {
        Self {
            send_lock: Arc::new(Mutex::new(())),
            deduped_events: Arc::new(DashMap::new()),
        }
    }
}

pub trait SendWithDedup {
    async fn send_with_dedup(&self, content: impl MessageLikeEventContent) -> Result<SendMessageLikeEventResult, Error>;

    async fn is_event_deduped(&self, event_id: &EventId) -> bool;
}

impl SendWithDedup for Room {
    async fn send_with_dedup(&self, content: impl MessageLikeEventContent) -> Result<SendMessageLikeEventResult, Error> {
        let room_id = self.room_id().to_owned();

        let state = DEDUP_STATE
            .entry(room_id.clone())
            .or_insert_with(RoomDedupState::new)
            .clone();

        let _guard = state.send_lock.lock().await;

        let result = self.send(content).await?;
        let event_id = result.response.event_id.to_owned();

        state.deduped_events.insert(event_id, ());

        Ok(result)
    }

    async fn is_event_deduped(&self, event_id: &EventId) -> bool {
        let room_id = self.room_id();

        if let Some(state) = DEDUP_STATE.get(room_id) {
            let _guard = state.send_lock.lock().await;

            let deduped = state.deduped_events.contains_key(event_id);
            if deduped {
                state.deduped_events.remove(event_id);
            }
            deduped
        } else {
            false
        }
    }
}