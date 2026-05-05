use crate::notification::circle_store::CircleStore;
use crate::notification::models::notification_handle::NotificationHandle;
use crate::notification::models::soap_holder::SoapHolder;
use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::services::session::contact_list_service::{ContactListService, ContactListServiceImpl};
use crate::tachyon::services::session::contact_service::{ContactService, ContactServiceImpl};
use crate::tachyon::services::session::incoming_message_service::{IncomingMessagingService, IncomingMessagingServiceImpl};
use crate::tachyon::services::session::outgoing_message_service::{OutgoingMessageService, OutgoingMessageServiceImpl};
use crate::tachyon::services::session::user_service::{UserService, UserServiceImpl};
use dashmap::DashMap;
use matrix_sdk::ruma::OwnedRoomId;
use msnp::shared::models::msn_user::MsnUser;
use std::sync::{Arc, Mutex, RwLock};
use msnp::msnp::models::contact_list::ContactList;
use crate::tachyon::state::session::room_proxy_repository::RoomProxyRepository;
use crate::tachyon::state::session::tachyon_client_repository::TachyonSessionData;

#[derive(Clone)]
pub struct TachyonClient {

    contact_list_service: Arc<dyn ContactListService>,
    contact_service: Arc<dyn ContactService>,
    incoming_message_service: Arc<dyn IncomingMessagingService>,
    outgoing_message_service: Arc<dyn OutgoingMessageService>,
    user_service: Arc<dyn UserService>,
    session_data: TachyonSessionData,
}

impl TachyonClient {
    pub(crate) fn from_session_data(session_data: TachyonSessionData) -> Self {
        let contact_list_service = Arc::new(ContactListServiceImpl::new(
            Arc::new(Mutex::new(ContactList::default()))
        ));

        let room_proxies_repository = Arc::new(session_data.clone()) as Arc<dyn RoomProxyRepository>;

        let own_user = Arc::new(RwLock::new(session_data.own_user()));

        let user_service = Arc::new(UserServiceImpl::new(session_data.matrix_client().to_owned(), room_proxies_repository,own_user));

        let contact_service = Arc::new(ContactServiceImpl::new(Default::default(), Default::default(), session_data.matrix_client().to_owned(),  user_service.clone()));

        let incoming_message_service = Arc::new(IncomingMessagingServiceImpl::new(Arc::new(session_data.clone()), user_service.clone()));

        let outgoing_message_service = Arc::new(OutgoingMessageServiceImpl{});
        
        Self {
            contact_list_service,
            contact_service,
            incoming_message_service,
            outgoing_message_service,
            user_service,
            session_data,
        }
    }
}

impl TachyonClient {
    pub fn new(
        contact_list_service: Arc<dyn ContactListService>,
        contact_service: Arc<dyn ContactService>,
        incoming_message_service: Arc<dyn IncomingMessagingService>,
        outgoing_message_service: Arc<dyn OutgoingMessageService>,
        user_service: Arc<dyn UserService>,
        session_data: TachyonSessionData
    ) -> Self {
       Self {

            contact_list_service,
            contact_service,
            incoming_message_service,
            outgoing_message_service,
            user_service,
            session_data
        }
    }

    pub fn contact_list(&self) -> Arc<dyn ContactListService>{
        self.contact_list_service.clone()
    }

    pub fn contacts(&self) -> Arc<dyn ContactService> {
        self.contact_service.clone()
    }

    pub fn incoming_messages(&self) -> Arc<dyn IncomingMessagingService> {
        self.incoming_message_service.clone()
    }

    pub fn outgoing_messages(&self) -> Arc<dyn OutgoingMessageService> {
        self.outgoing_message_service.clone()
    }

    pub fn users(&self) -> Arc<dyn UserService> {
        self.user_service.clone()
    }

    // --- Legacy accessors (delegate to session_data) ---
    // TODO: Replace call sites with service-based accessors and remove these

    pub fn own_user(&self) -> MsnUser {
        self.user_service.own_user()
    }

    pub fn shutdown(&self) {
        self.session_data.shutdown();
    }

    pub fn soap_holder(&self) -> &SoapHolder {
        self.session_data.soap_holder()
    }

    pub fn notification_handle(&self) -> NotificationHandle {
        self.session_data.notification_handle()
    }

    pub fn alerts(&self) -> &DashMap<i32, crate::tachyon::alert::Alert> {
        self.session_data.alerts()
    }

    pub fn circle_store(&self) -> &CircleStore {
        &self.session_data.session_data.circle_store
    }

    pub fn switchboards(&self) -> &DashMap<OwnedRoomId, SwitchboardHandle> {
        &self.session_data.session_data.switchboards
    }

    pub fn matrix_client(&self) -> &crate::matrix::MatrixClient {
        self.session_data.matrix_client()
    }

    pub fn ticket_token(&self) -> msnp::shared::models::ticket_token::TicketToken {
        self.session_data.ticket_token()
    }

    pub fn get_contact_list(
        &self,
    ) -> &std::sync::Mutex<msnp::msnp::models::contact_list::ContactList> {
        self.session_data.get_contact_list()
    }

    pub fn own_user_mut(&self) -> std::sync::RwLockWriteGuard<'_, MsnUser> {
        self.session_data.own_user_mut()
    }
}
