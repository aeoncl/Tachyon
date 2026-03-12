use std::net::Ipv4Addr;
use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use tokio::sync::mpsc;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use matrix_sdk::{Client, SlidingSync};
use dashmap::DashMap;
use matrix_sdk::ruma::{OwnedRoomId, RoomId};
use anyhow::anyhow;
use log::{debug, error, trace};
use tokio_retry2::{Retry, RetryError};
use tokio_retry2::strategy::{ExponentialBackoff, MaxInterval};
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::notification::command::rng::RngServer;
use msnp::msnp::notification::models::ip_address::IpAddress;
use msnp::msnp::switchboard::models::session_id::SessionId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use crate::notification::circle_store::CircleStore;
use crate::tachyon::client_store::ClientStoreError;
use crate::notification::models::notification_handle::NotificationHandle;
use crate::notification::models::soap_holder::SoapHolder;
use crate::switchboard::models::switchboard_handle::{SwitchboardHandle, SwitchboardState};
use crate::switchboard::models::switchboard_token::SwitchboardToken;

pub struct ClientDataInner {
    pub own_user: Mutex<MsnUser>,
    pub ticket_token: TicketToken,
    pub matrix_client: Client,
    pub sliding_sync: SlidingSync,
    pub contact_list: Mutex<ContactList>,
    pub soap_holder: SoapHolder,
    pub switchboards: DashMap<OwnedRoomId, SwitchboardHandle>,
    pub circle_store: CircleStore,
    pub notification_handle: NotificationHandle,
}

#[derive(Clone)]
pub struct TachyonClient {
    pub inner: Arc<ClientDataInner>
}

pub struct SwitchboardService {
    tachyon_client: TachyonClient
}

impl SwitchboardService {

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

    pub fn get_or_create(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle {
        if let Some(switchboard) = self.get(room_id) {
            return switchboard;
        }

        self.create(room_id, &inviter)
    }

    pub fn remove(&self, room_id: &RoomId) -> Option<SwitchboardHandle> {
        self.tachyon_client.inner.switchboards
            .remove(room_id)
            .map(|(room_id, sb)| sb)
    }

    pub fn create(&self, room_id: &RoomId, inviter: &MsnUser) -> SwitchboardHandle {

        let switchboard = SwitchboardHandle::new(SessionId::random(), room_id.to_owned());
        self.tachyon_client.inner.switchboards.insert(room_id.to_owned(), switchboard.clone());

        let switchboard_handle_clone = switchboard.clone();
        let notification_client_clone = self.tachyon_client.notification_handle().clone();

        let inviter_clone = inviter.clone();
        let token = self.tachyon_client.ticket_token().0;
        let room_id = room_id.to_owned();
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
                    .max_delay_millis(1000)
                    .max_interval(5000)
                    .take(5);

                let switchboard_handle_clone_clone = switchboard_handle_clone.clone();
                let result = Retry::spawn(retry_strategy, move || {
                    let switchboard_clone = switchboard_handle_clone_clone.clone();
                    async move {
                        Self::check_sb_initialized(switchboard_clone)
                    }
                }
                ).await;

                match result {
                    Ok(_) => {
                        debug!("Switchboard {} is initialized", &switchboard_handle_clone.room_id);
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

impl TachyonClient {
    pub fn new(
        user: MsnUser,
        token: TicketToken,
        notification_sender: mpsc::Sender<NotificationServerCommand>,
        matrix_client: Client,
        sliding_sync: SlidingSync,
    ) -> TachyonClient {
        TachyonClient {
            inner: Arc::new(ClientDataInner {
                own_user: Mutex::new(user),
                ticket_token: token,
                matrix_client,
                sliding_sync,
                contact_list: Default::default(),
                soap_holder: Default::default(),
                switchboards: Default::default(),
                circle_store: CircleStore::new(),
                notification_handle: NotificationHandle::new(notification_sender),
            })
        }
    }

    pub fn switchboards(&self) -> SwitchboardService {
        SwitchboardService::new(self.clone())
    }

    pub fn own_user(&self) -> Result<MsnUser, ClientStoreError> {
        Ok(self.own_user_mut()?.clone())
    }

    pub fn own_user_mut(&self) -> Result<MutexGuard<'_, MsnUser>, ClientStoreError> {
        Ok(self
            .inner
            .own_user
            .lock()
            .map_err(|e| ClientStoreError::PoisonnedLockError {
                name: "User".into(),
                source: anyhow!(e.to_string()),
            })?)
    }

    pub fn soap_holder(&self) -> &SoapHolder {
        &self.inner.soap_holder
    }

    pub fn get_contact_list(&self) -> &Mutex<ContactList> {
        &self.inner.contact_list
    }

    //  pub fn get_contact_service(&self) -> ContactService {
    //       self.contact_service.clone()
    //  }

    pub fn ticket_token(&self) -> TicketToken {
        self.inner.ticket_token.clone()
    }

    pub fn matrix_client(&self) -> Client {
        self.inner.matrix_client.clone()
    }

    pub fn sliding_sync(&self) -> SlidingSync {
        self.inner.sliding_sync.clone()
    }

    pub fn notification_handle(&self) -> NotificationHandle {
        self.inner.notification_handle.clone()
    }
}