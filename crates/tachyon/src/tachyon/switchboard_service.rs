use std::net::Ipv4Addr;
use matrix_sdk::ruma::RoomId;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::rng::RngServer;
use msnp::msnp::notification::models::ip_address::IpAddress;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::msn_user::MsnUser;
use tokio_retry2::strategy::{ExponentialBackoff, MaxInterval};
use tokio_retry2::{Retry, RetryError};
use log::{debug, error, trace};
use anyhow::anyhow;
use crate::switchboard::models::switchboard_handle::{SwitchboardHandle, SwitchboardState};
use crate::switchboard::models::switchboard_token::SwitchboardToken;
use crate::tachyon::tachyon_client::TachyonClient;

#[derive(Clone)]
pub struct SwitchboardService {
    tachyon_client: TachyonClient
}

impl SwitchboardService {

    //TODO Better locks on switchboard.
    pub fn new(tachyon_client: TachyonClient) -> Self {
        SwitchboardService {
            tachyon_client,
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

        let switchboard_handle_clone = switchboard.clone();
        let notification_client_clone = self.tachyon_client.notification_handle().clone();

        let inviter_clone = inviter.clone();
        let token = self.tachyon_client.ticket_token().0;
        let room_id = room_id.to_owned();

        let sb_service_clone = self.tachyon_client.switchboards();
        //TODO make a task that checks on all SBs instead of just one.

        tokio::spawn(async move {
            let max_tries = 3;
            for current_try in 0..max_tries {
                let _ = notification_client_clone.send(
                    NotificationServerCommand::RNG(
                        RngServer::new(
                            SessionId::random(),
                            IpAddress::new(Ipv4Addr::new(127, 0, 0, 1), 1864),
                            SwitchboardToken::new(room_id.clone(), token.clone()).into(),
                            inviter_clone.get_email_address().clone(),
                            inviter_clone.compute_display_name().to_string()
                        )
                    )
                ).await;

                let retry_strategy = ExponentialBackoff::from_millis(10)
                    .factor(1)
                    .max_delay_millis(50000)
                    .max_interval(5000)
                    .take(10);

                let room_id_clone = room_id.clone();
                let sb_service_clone_clone = sb_service_clone.clone();

                let result = Retry::spawn(retry_strategy, move || {
                    let sb_service_clone = sb_service_clone_clone.clone();
                    let switchboard = sb_service_clone.get(&room_id_clone).unwrap();
                    async move {
                        Self::check_sb_initialized(switchboard.clone())
                    }
                }
                ).await;

                match result {
                    Ok(_) => {
                        debug!("Switchboard {} is initialized", &switchboard_handle_clone.room_id);
                        switchboard_handle_clone.send_pending_events().await;
                        return;
                    }
                    Err(e) => {
                        debug!("Failed to initialize Switchboard for room: {} (try {} of {})", &switchboard_handle_clone.room_id, current_try, max_tries);
                        trace!("Switchboard initialization error: {:?}", e);
                    }
                }
            }

            error!("Failed to initialize Switchboard for room: {}, killing client.", &switchboard_handle_clone.room_id);
            let _ = notification_client_clone.send(
                NotificationServerCommand::OUT
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