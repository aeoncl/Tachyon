use crate::matrix::MatrixClient;
use crate::notification::circle_store::CircleStore;
use crate::notification::models::notification_handle::NotificationHandle;
use crate::notification::models::soap_holder::SoapHolder;
use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::alert::Alert;
use crate::tachyon::client::switchboards::SwitchboardService;
use crate::tachyon::global::tachyon_config::TachyonConfig;
use dashmap::DashMap;
use matrix_sdk::locks::RwLock;
use matrix_sdk::ruma::OwnedRoomId;
use msnp::msnp::models::contact_list::ContactList;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use std::sync::{Arc, Mutex, RwLockWriteGuard};
use tokio::sync::broadcast;

pub struct TachyonSessionData {
    pub matrix_client: MatrixClient,
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
    pub session_data: Arc<TachyonSessionData>
}

impl TachyonClient {
    pub fn new(
        matrix_client: MatrixClient,
        config: TachyonConfig,
        user: MsnUser,
        token: TicketToken,
        notification_handle: NotificationHandle,
        client_shutdown_snd: broadcast::Sender<()>,
        client_shutdown_recv: broadcast::Receiver<()>,
    ) -> TachyonClient {
        TachyonClient {
            session_data: Arc::new(TachyonSessionData {
                matrix_client,
                own_user: RwLock::new(user),
                ticket_token: token,
                contact_list: Default::default(),
                soap_holder: Default::default(),
                switchboards: Default::default(),
                alerts: Default::default(),
                circle_store: CircleStore::new(),
                notification_handle,
                config,
                client_shutdown_snd,
                client_shutdown_recv,
            })
        }
    }

    pub fn switchboards(&self) -> Box<dyn SwitchboardService> {
        Box::new(self.clone()) as Box<dyn SwitchboardService>
    }

    pub fn alerts(&self) -> &DashMap<i32, Alert> {
        &self.session_data.alerts
    }

    pub fn own_user(&self) -> MsnUser {
        self.session_data.own_user.read().clone()
    }

    pub fn own_user_mut(&self) -> RwLockWriteGuard<'_, MsnUser> {
        self.session_data.own_user.write()
    }

    pub fn soap_holder(&self) -> &SoapHolder {
        &self.session_data.soap_holder
    }

    pub fn get_contact_list(&self) -> &Mutex<ContactList> {
        &self.session_data.contact_list
    }

    pub fn ticket_token(&self) -> TicketToken {
        self.session_data.ticket_token.clone()
    }

    pub fn shutdown_sender(&self) -> broadcast::Sender<()> {
        self.session_data.client_shutdown_snd.clone()
    }

    pub fn shutdown(&self) {
        let _ = self.session_data.client_shutdown_snd.send(());
    }

    pub fn shutdown_receiver(&self) -> broadcast::Receiver<()> {
        self.session_data.client_shutdown_recv.resubscribe()
    }

    pub fn notification_handle(&self) -> NotificationHandle {
        self.session_data.notification_handle.clone()
    }

    //TODO: Remove this
    pub fn matrix_client(&self) -> &MatrixClient {
        &self.session_data.matrix_client
    }
}