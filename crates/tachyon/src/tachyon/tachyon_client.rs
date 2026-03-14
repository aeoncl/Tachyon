use crate::notification::circle_store::CircleStore;
use crate::notification::models::notification_handle::NotificationHandle;
use crate::notification::models::soap_holder::SoapHolder;
use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::client_store::ClientStoreError;
use crate::tachyon::switchboard_service::SwitchboardService;
use anyhow::anyhow;
use dashmap::DashMap;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::{Client, SlidingSync};
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::mpsc;

pub struct TachyonClientInner {
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
    pub inner: Arc<TachyonClientInner>
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
            inner: Arc::new(TachyonClientInner {
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