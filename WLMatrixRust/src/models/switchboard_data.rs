use std::sync::{Mutex, RwLock, Arc};

use matrix_sdk::ruma::events::{room::message::{RoomMessageEventContent}, OriginalSyncMessageLikeEvent};
use tokio::sync::broadcast::{Sender, self, Receiver};

pub struct SwitchboardData {
    pub sender: Sender<String>,
    pub receiver: Arc<Mutex<Option<Receiver<String>>>>
}

impl SwitchboardData {

    pub fn new() -> SwitchboardData {

        let (sender, receiver) = broadcast::channel::<String>(30);
        return SwitchboardData{ sender: sender, receiver: Arc::new(Mutex::new(Some(receiver))) };
    }
}