use crate::switchboard::models::switchboard_handle::{SwitchboardHandle, SwitchboardState};
use crate::switchboard::models::switchboard_token::SwitchboardToken;
use crate::tachyon::client::tachyon_client::TachyonClient;
use anyhow::anyhow;
use matrix_sdk::ruma::RoomId;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::rng::RngServer;
use msnp::msnp::notification::models::ip_address::IpAddress;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::msn_user::MsnUser;
use std::net::Ipv4Addr;
use tokio_retry2::RetryError;

#[derive(Clone)]
pub struct SwitchboardService {
    tachyon_client: TachyonClient,
    switchboard_port: u32
}

impl SwitchboardService {

    //TODO Better locks on switchboard.
    pub fn new(tachyon_client: TachyonClient, switchboard_port: u32) -> Self {
        SwitchboardService {
            tachyon_client,
            switchboard_port,
        }
    }

    pub fn get(&self, room_id: &RoomId) -> Option<SwitchboardHandle> {
        self.tachyon_client.inner.switchboards.get(room_id).map(|sb| sb.clone())
    }

    pub fn contains(&self, room_id: &RoomId) -> bool {
        self.tachyon_client.inner.switchboards.contains_key(room_id)
    }

    pub fn insert(&self, switchboard: SwitchboardHandle) -> Result<SwitchboardHandle, anyhow::Error> {
        {
            let mut pending_events = switchboard.pending_events.lock().map_err(|e| anyhow::anyhow!("Failed to lock pending events: {}", e))?;
            let old_value = self.tachyon_client.inner.switchboards.insert(switchboard.room_id.clone(), switchboard.clone());
            if let Some(old) = old_value {
                for old_pending in old.pending_events.lock().map_err(|e| anyhow::anyhow!("Failed to lock pending events: {}", e))?.drain(..) {
                    pending_events.push(old_pending);
                }

            }
        }
        Ok(switchboard)
    }

    pub fn get_or_initialize(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle {
        if let Some(switchboard) = self.get(room_id) {
            return switchboard;
        }

        self.initialize(room_id, &inviter)
    }

    pub fn remove(&self, room_id: &RoomId) -> Option<SwitchboardHandle> {
        self.tachyon_client.inner.switchboards
            .remove(room_id)
            .map(|(room_id, sb)| sb)
    }

    pub fn initialize(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle {

        let switchboard = SwitchboardHandle::new(SessionId::random(), room_id.to_owned());
        self.tachyon_client.inner.switchboards.insert(room_id.to_owned(), switchboard.clone());

        let notification_client_clone = self.tachyon_client.notification_handle().clone();

        let inviter_clone = inviter.clone();
        let token = self.tachyon_client.ticket_token().0;
        let room_id = room_id.to_owned();

        let switchboard_port = self.switchboard_port;
        tokio::spawn(async move {
            let _ = notification_client_clone.send(
                NotificationServerCommand::RNG(
                    RngServer::new(
                        SessionId::random(),
                        IpAddress::new(Ipv4Addr::new(127, 0, 0, 1), switchboard_port),
                        SwitchboardToken::new(room_id.clone(), token.clone()).into(),
                        inviter_clone.get_email_address().clone(),
                        inviter_clone.compute_display_name().to_string()
                    )
                )
            ).await;
        });

        switchboard
    }


    fn check_sb_initialized(handle: SwitchboardHandle) -> Result<(), RetryError<anyhow::Error>> {
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