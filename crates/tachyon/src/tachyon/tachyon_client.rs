use crate::tachyon::services::session::contact_list_service::ContactListService;
use crate::tachyon::services::session::contact_service::ContactService;
use crate::tachyon::services::session::incoming_message_service::IncomingMessagingService;
use crate::tachyon::services::session::outgoing_message_service::OutgoingMessageService;
use crate::tachyon::services::session::user_service::UserService;
use std::sync::Arc;

#[derive(Clone)]
pub struct TachyonClient {

    contact_list_service: Arc<dyn ContactListService>,
    contact_service: Arc<dyn ContactService>,
    incoming_message_service: Arc<dyn IncomingMessagingService>,
    outgoing_message_service: Arc<dyn OutgoingMessageService>,
    user_service: Arc<dyn UserService>,

}

impl TachyonClient {
    pub fn new(
        contact_list_service: Arc<dyn ContactListService>,
        contact_service: Arc<dyn ContactService>,
        incoming_message_service: Arc<dyn IncomingMessagingService>,
        outgoing_message_service: Arc<dyn OutgoingMessageService>,
        user_service: Arc<dyn UserService>,
    ) -> Self {
       Self {
            contact_list_service,
            contact_service,
            incoming_message_service,
            outgoing_message_service,
            user_service
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
}