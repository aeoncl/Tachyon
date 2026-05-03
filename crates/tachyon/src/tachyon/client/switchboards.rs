use crate::switchboard::models::switchboard_handle::{SwitchboardHandle, SwitchboardState};
use crate::switchboard::models::switchboard_token::SwitchboardToken;
use crate::tachyon::client::tachyon_client::TachyonClient;
use anyhow::anyhow;
use matrix_sdk::ruma::RoomId;
use msnp::msnp::notification::models::ip_address::IpAddress;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::msn_user::MsnUser;
use std::net::Ipv4Addr;
use tokio_retry2::RetryError;

pub trait SwitchboardService : Send + Sync {
    fn get(&self, room_id: &RoomId) -> Option<SwitchboardHandle>;
    fn contains(&self, room_id: &RoomId) -> bool;
    fn insert(&self, switchboard: SwitchboardHandle) -> Result<SwitchboardHandle, anyhow::Error>;
    fn get_or_initialize(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle;
    fn remove(&self, room_id: &RoomId) -> Option<SwitchboardHandle>;
    fn initialize(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle;
    fn check_sb_initialized(&self, handle: SwitchboardHandle) -> Result<(), RetryError<anyhow::Error>>;

    }

impl SwitchboardService for TachyonClient {

    fn get(&self, room_id: &RoomId) -> Option<SwitchboardHandle> {
        self.session_data.switchboards.get(room_id).map(|sb| sb.clone())
    }

    fn contains(&self, room_id: &RoomId) -> bool {
        self.session_data.switchboards.contains_key(room_id)
    }

    fn insert(&self, switchboard: SwitchboardHandle) -> Result<SwitchboardHandle, anyhow::Error> {
        {
            let mut pending_events = switchboard.pending_events.lock().map_err(|e| anyhow::anyhow!("Failed to lock pending events: {}", e))?;
            let old_value = self.session_data.switchboards.insert(switchboard.room_id.clone(), switchboard.clone());
            if let Some(old) = old_value {
                for old_pending in old.pending_events.lock().map_err(|e| anyhow::anyhow!("Failed to lock pending events: {}", e))?.drain(..) {
                    pending_events.push(old_pending);
                }

            }
        }
        Ok(switchboard)
    }

    fn get_or_initialize(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle {
        if let Some(switchboard) = self.get(room_id) {
            return switchboard;
        }

        self.initialize(room_id, &inviter)
    }

    fn remove(&self, room_id: &RoomId) -> Option<SwitchboardHandle> {
        self.session_data.switchboards
            .remove(room_id)
            .map(|(room_id, sb)| sb)
    }

    fn initialize(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle {

        let switchboard = SwitchboardHandle::new(SessionId::random(), room_id.to_owned());
        self.session_data.switchboards.insert(room_id.to_owned(), switchboard.clone());

        let notification_handle = self.notification_handle().clone();
        let inviter_clone = inviter.to_owned();
        let switchboard_token = SwitchboardToken::new(room_id.to_owned(), self.ticket_token().0);
        let switchboard_port = self.session_data.config.switchboard_port;

        tokio::spawn(async move {
            let _ = notification_handle.request_switchboard(IpAddress::new(Ipv4Addr::new(127, 0, 0, 1), switchboard_port), switchboard_token.into(), inviter_clone).await;
        });

        switchboard
    }


    fn check_sb_initialized(&self, handle: SwitchboardHandle) -> Result<(), RetryError<anyhow::Error>> {
        match handle.switchboard_state.lock() {
            Ok(state) => {
                if let SwitchboardState::Ready {..} = &*state{
                    Ok(())
                } else {
                    RetryError::to_transient(anyhow!("Switchboard not ready"))
                }
            }
            Err(lock_err) => {
                RetryError::to_permanent(anyhow!("{}", lock_err))
            }
        }
    }

}