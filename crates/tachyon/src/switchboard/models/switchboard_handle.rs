use anyhow::anyhow;
use matrix_sdk::ruma::OwnedRoomId;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::models::session_id::SessionId;
use std::mem;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct SwitchboardHandle {
    pub session_id: SessionId,
    pub room_id: OwnedRoomId,
    pub pending_events: Arc<Mutex<Vec<SwitchboardServerCommand>>>,
    pub switchboard_state: Arc<Mutex<SwitchboardState>>
}


impl SwitchboardHandle {
    pub fn new(session_id: SessionId, room_id: OwnedRoomId) -> Self {
        Self {
            session_id,
            room_id,
            pending_events: Arc::new(Mutex::new(vec![])),
            switchboard_state: Arc::new(Mutex::new(SwitchboardState::default()))
        }
    }

    pub fn state(&self) -> Result<SwitchboardState, anyhow::Error> {
        Ok(self.switchboard_state.lock().map_err(|e| anyhow::anyhow!("Failed to acquire switchboard state lock: {}", e))?.clone())
    }

    pub fn set_state(&mut self, state: SwitchboardState) -> Result<(), anyhow::Error> {

        let mut lock = self.switchboard_state.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        let _ = mem::replace(&mut *lock, state);
        Ok(())
    }

    pub async fn send_command(&self, command: SwitchboardServerCommand) -> Result<(), anyhow::Error> {

        let state = self.state().map_err(|e| anyhow!(e))?;

        match state {
            SwitchboardState::Initializing => {
                self.pending_events.lock().map_err(|e| anyhow!("Lock error: {}", e))?.push(command);
                Ok(())
            }
            SwitchboardState::Ready {
                msnp_sender, p2p_sender
            } => {
                msnp_sender.send(command).await?;
                Ok(())
            }
        }
    }
}


#[derive(Clone)]
pub enum SwitchboardState {
    Initializing,
    Ready {
        msnp_sender: mpsc::Sender<SwitchboardServerCommand>,
        p2p_sender: mpsc::Sender<String>,
    }
}

impl Default for SwitchboardState {
    fn default() -> Self {
        Self::Initializing
    }
}
