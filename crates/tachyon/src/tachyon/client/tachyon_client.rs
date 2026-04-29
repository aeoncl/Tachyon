use crate::notification::circle_store::CircleStore;
use crate::notification::models::notification_handle::NotificationHandle;
use crate::notification::models::soap_holder::SoapHolder;
use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::alert::Alert;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::switchboard_service::SwitchboardService;
use dashmap::DashMap;
use matrix_sdk::locks::RwLock;
use matrix_sdk::ruma::OwnedRoomId;
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::{Arc, Mutex, RwLockWriteGuard};
use tokio::sync::{broadcast, mpsc};

pub struct TachyonClientInner {
    pub own_user: RwLock<MsnUser>,
    pub ticket_token: TicketToken,
    pub contact_list: Mutex<ContactList>,
    pub soap_holder: SoapHolder,
    pub switchboards: DashMap<OwnedRoomId, SwitchboardHandle>,
    pub alerts: DashMap<i32, Alert>,
    pub circle_store: CircleStore,
    pub notification_handle: NotificationHandle,
    pub config: TachyonConfig,
    pub client_shutdown_snd: broadcast::Sender<()>,
    pub client_shutdown_recv: broadcast::Receiver<()>,
}

#[derive(Clone)]
pub struct TachyonClient {
    pub inner: Arc<TachyonClientInner>
}

impl TachyonClient {
    pub fn new(
        config: TachyonConfig,
        user: MsnUser,
        token: TicketToken,
        notification_sender: mpsc::Sender<NotificationServerCommand>,
        client_shutdown_snd: broadcast::Sender<()>,
        client_shutdown_recv: broadcast::Receiver<()>,
    ) -> TachyonClient {
        TachyonClient {
            inner: Arc::new(TachyonClientInner {
                own_user: RwLock::new(user),
                ticket_token: token,
                contact_list: Default::default(),
                soap_holder: Default::default(),
                switchboards: Default::default(),
                alerts: Default::default(),
                circle_store: CircleStore::new(),
                notification_handle: NotificationHandle::new(notification_sender),
                config,
                client_shutdown_snd,
                client_shutdown_recv,
            })
        }
    }

    pub fn switchboards(&self) -> SwitchboardService {
        SwitchboardService::new(self.clone(), self.inner.config.switchboard_port)
    }

    pub fn alerts(&self) -> &DashMap<i32, Alert> {
        &self.inner.alerts
    }

    pub fn own_user(&self) -> MsnUser {
        self.inner.own_user.read().clone()
    }

    pub fn own_user_mut(&self) -> RwLockWriteGuard<'_, MsnUser> {
        self.inner.own_user.write()
    }

    pub fn soap_holder(&self) -> &SoapHolder {
        &self.inner.soap_holder
    }

    pub fn get_contact_list(&self) -> &Mutex<ContactList> {
        &self.inner.contact_list
    }

    pub fn ticket_token(&self) -> TicketToken {
        self.inner.ticket_token.clone()
    }

    pub fn shutdown_sender(&self) -> broadcast::Sender<()> {
        self.inner.client_shutdown_snd.clone()
    }

    pub fn shutdown(&self) {
        let _ = self.inner.client_shutdown_snd.send(());
    }

    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.inner.client_shutdown_recv.resubscribe()
    }

    pub fn notification_handle(&self) -> NotificationHandle {
        self.inner.notification_handle.clone()
    }
}