//! The single dedup path (§3.5, H7). Pre-D3 this was a process-global
//! lazily-initialized dedup map keyed by room only — unowned, never reset,
//! shared across accounts, so a second account in the same room had its
//! sends suppressed and its echoes misattributed (§1.4 H7). The state is now
//! an owned field of `TachyonClientInner`: its lifetime is exactly the
//! signed-in account's lifetime, and the cross-account bug dies by
//! ownership, not by a compound key.
//!
//! This module also owns `RoomOp` and its executor — the ONE Matrix-bound
//! text-send path that every §1.1 site routes through, which is what retires
//! the uum echo gap instead of patching it.
use crate::tachyon::client::tachyon_client::TachyonClient;
use dashmap::DashMap;
use matrix_sdk::room::futures::{SendMessageLikeEvent, SendMessageLikeEventResult};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::{EventId, OwnedEventId, OwnedRoomId, RoomId};
use matrix_sdk::{Error, Room};
use std::sync::Arc;
use tokio::sync::Mutex;

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

/// Owned per-account dedup state — the pre-D3 process-global DashMap, with
/// the same per-room send-lock + event-id table semantics, minus the
/// process-global lifetime.
#[derive(Default)]
pub struct DedupState {
    rooms: DashMap<OwnedRoomId, RoomDedupState>,
}

impl DedupState {
    fn room_state(&self, room_id: &RoomId) -> RoomDedupState {
        self.rooms
            .entry(room_id.to_owned())
            .or_insert_with(RoomDedupState::new)
            .clone()
    }

    /// The send half of the single path: serialize per room, send, then
    /// record the event id so the sender's own echo is suppressed —
    /// the pre-D3 dedup-send trait semantics, owned.
    pub async fn send_text(
        &self,
        room: &Room,
        content: RoomMessageEventContent,
    ) -> Result<SendMessageLikeEventResult, Error> {
        let state = self.room_state(room.room_id());

        let _guard = state.send_lock.lock().await;

        let result = room.send(content).await?;
        let event_id = result.response.event_id.to_owned();

        state.deduped_events.insert(event_id, ());

        Ok(result)
    }

    /// The inbound half of the single path: single-use echo check — a
    /// deduped event id is consumed (removed), exactly as pre-D3.
    pub async fn is_event_deduped(&self, room_id: &RoomId, event_id: &EventId) -> bool {
        if let Some(state) = self.rooms.get(room_id) {
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

/// The Matrix-bound op vocabulary (§3.2). Bottom-up, escape-hatch rule
/// applied ruthlessly: `SendText` is the ONLY variant because it is the only
/// one with a demonstrated difference — §1.1's four divergent text-send
/// paths, one of which (uum) bypassed dedup entirely. The sketch's
/// `Invite`/`Typing` variants join when a real call site demands one; a
/// variant nobody constructs is §6 D ceremony. The room rides as the SDK
/// handle every D3 call site already holds (the sketch's `OwnedRoomId`
/// would force a re-lookup and a new failure mode for zero benefit).
pub enum RoomOp {
    SendText { room: Room, content: RoomMessageEventContent },
}

/// apply() (§3): the single executor for Matrix-bound ops — the only place
/// an event-handler-front text send touches the SDK. Every §1.1 site routes
/// through here, so there is exactly one dedup path.
pub async fn apply_room_op(tachyon_client: &TachyonClient, op: RoomOp) -> Result<(), Error> {
    match op {
        RoomOp::SendText { room, content } => {
            tachyon_client.dedup().send_text(&room, content).await.map(|_| ())
        }
    }
}

#[cfg(test)]
mod tests {
    use matrix_sdk::ruma::{event_id, room_id};

    use super::DedupState;

    #[tokio::test]
    async fn a_room_without_recorded_sends_never_dedupes_an_inbound_event() {
        // The owned state starts empty per account — the single-use inbound
        // check answers false instead of touching any process global.
        let dedup = DedupState::default();

        assert!(!dedup.is_event_deduped(room_id!("!abc123:test.local"), event_id!("$ev1:test.local")).await);
    }
}
