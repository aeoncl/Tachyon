use std::sync::Mutex;
use dashmap::DashMap;
use lazy_static::lazy_static;
use matrix_sdk::{Error, Room};
use matrix_sdk::room::futures::{SendMessageLikeEvent, SendMessageLikeEventResult};
use matrix_sdk::ruma::events::MessageLikeEventContent;
use matrix_sdk::ruma::{EventId, OwnedEventId, RoomId, OwnedRoomId};

lazy_static! {
    static ref DEDUP_EVENTS: DashMap<DedupId, ()> = DashMap::new();
}

#[derive(Eq, Hash, PartialEq)]
struct DedupId {
    event_id: OwnedEventId,
    room_id: OwnedRoomId
}

impl DedupId {
    fn new(event_id: OwnedEventId, room_id: OwnedRoomId) -> Self {
        Self {
            event_id,
            room_id
        }
    }

    fn  new_from_ref(event_id: &EventId, room_id: &RoomId) -> Self {
        Self {
            event_id: event_id.to_owned(),
            room_id: room_id.to_owned()
        }
    }
}

pub trait SendWithDedup {
    async fn send_with_dedup(&self, content: impl MessageLikeEventContent) -> Result<SendMessageLikeEventResult, Error>;

    fn is_event_deduped(&self, event_id: &EventId) -> bool;
}

impl SendWithDedup for Room {
    async fn send_with_dedup(&self, content: impl MessageLikeEventContent) -> Result<SendMessageLikeEventResult, Error> {
        let result = self.send(content).await?;
        DEDUP_EVENTS.insert(DedupId::new_from_ref(result.response.event_id.as_ref(), self.room_id()), ());
        Ok(result)
    }

    fn is_event_deduped(&self, event_id: &EventId) -> bool {
        let dedup_id = DedupId::new_from_ref(event_id, self.room_id());
        let deduped = DEDUP_EVENTS.contains_key(&dedup_id);
        if deduped {
            DEDUP_EVENTS.remove(&dedup_id);
        }

        deduped
    }
}
