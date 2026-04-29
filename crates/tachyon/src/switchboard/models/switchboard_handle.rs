use anyhow::anyhow;
use matrix_sdk::ruma::OwnedRoomId;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::msnp::switchboard::models::session_id::SessionId;
use std::mem;
use std::sync::{Arc, Mutex};
use matrix_sdk::Room;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct SwitchboardHandle {
    pub session_id: SessionId,
    pub room_id: OwnedRoomId,
    pub pending_events: Arc<Mutex<Vec<SwitchboardServerCommand>>>,
    pub switchboard_state: Arc<Mutex<SwitchboardState>>
}

impl SwitchboardHandle {
    pub(crate) async fn send_pending_events(&self) -> Result<(), anyhow::Error>  {
        if let SwitchboardState::Ready { msnp_sender } = self.state()? {

            let events: Vec<SwitchboardServerCommand> = {
                self.pending_events.lock().map_err(|e| anyhow!("Failed to acquire pending events lock: {}", e))?.drain(..).collect()
            };

            for event in events {
                msnp_sender.send(event).await?;
            }
        }
        Ok(())
    }
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

    pub fn new_ready(session_id: SessionId, room_id: OwnedRoomId, msnp_sender: mpsc::Sender<SwitchboardServerCommand>) -> Self {
        Self {
            session_id,
            room_id,
            pending_events: Arc::new(Mutex::new(vec![])),
            switchboard_state: Arc::new(Mutex::new(SwitchboardState::Ready { msnp_sender }))
        }
    }

    pub fn state(&self) -> Result<SwitchboardState, anyhow::Error> {
        Ok(self.switchboard_state.lock().map_err(|e| anyhow::anyhow!("Failed to acquire switchboard state lock: {}", e))?.clone())
    }

    pub async fn set_state(&mut self, state: SwitchboardState) -> Result<(), anyhow::Error> {

        let send_events = matches!(&state, SwitchboardState::Ready {..});
        
        {
            let mut lock = self.switchboard_state.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
            let _ = mem::replace(&mut *lock, state);
        }

        if send_events {
            self.send_pending_events().await?
        }

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
                msnp_sender
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
        msnp_sender: mpsc::Sender<SwitchboardServerCommand>
    }
}

impl Default for SwitchboardState {
    fn default() -> Self {
        Self::Initializing
    }
}
