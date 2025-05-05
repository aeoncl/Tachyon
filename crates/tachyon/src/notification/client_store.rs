use std::collections::VecDeque;
use std::sync::{Arc, LockResult, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::notification::circle_store::CircleStore;
use anyhow::anyhow;
use dashmap::DashMap;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::Client;
use matrix_sdk::config::SyncSettings;
use matrix_sdk_ui::sync_service::{SyncService, SyncServiceBuilder};
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::oim::OIM;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::response::CircleData;
use msnp::soap::abch::msnab_datatypes::{BaseMember, ContactType};
use thiserror::Error;
use thiserror::__private::AsDynError;
use tokio::sync::mpsc;
use crate::matrix::direct_service::DirectService;

#[derive(Clone)]
pub struct SwitchboardHandle {
    room_id: String,
    msnp_sender: mpsc::Sender<SwitchboardServerCommand>,
    p2p_sender: mpsc::Sender<String>
}


pub enum Contact {
    Contact(ContactType),
    Circle(CircleData)
}

#[derive(Default)]
pub struct SoapHolder {
    pub oims: DashMap<String, OIM>,
    pub contacts: Mutex<Vec<Contact>>,
    pub circle_contacts: DashMap<String, Vec<ContactType>>,
    pub memberships: Mutex<VecDeque<BaseMember>>
}

pub struct ClientDataInner {
    pub user: RwLock<MsnUser>,
    pub ticket_token: TicketToken,
    pub matrix_client: Client,
    pub sync_service: SyncService,
    pub direct_service: DirectService,
    pub contact_list: Mutex<ContactList>,
    pub soap_holder: SoapHolder,
    pub switchboards: DashMap<OwnedRoomId, SwitchboardHandle>,
    pub circle_store: CircleStore
}


#[derive(Clone)]
pub struct ClientData {
    pub inner: Arc<ClientDataInner>
}

#[derive(Error, Debug)]
pub enum ClientStoreError {
    #[error("Mutex lock was poisonned: {}", .name)]
    PoisonnedLockError{name: String, source: anyhow::Error}
}

impl ClientData {
    pub fn new(user: MsnUser, token: TicketToken, matrix_client: Client, sync_service: SyncService, direct_service: DirectService) -> ClientData {
        
        ClientData{ inner: Arc::new(ClientDataInner {
            user: RwLock::new(user),
            ticket_token: token,
            matrix_client,
            sync_service,
            direct_service,
            contact_list: Default::default(),
            soap_holder: Default::default(),
            switchboards: Default::default(),
            circle_store: CircleStore::new(),
        })
        }
    }

    pub fn get_switchboard(&self, id: OwnedRoomId) -> Option<SwitchboardHandle> {
        match self.inner.switchboards.get(&id) {
            None => {
                None
            }
            Some(found) => {
                Some(found.value().clone())
            }
        }
    }

    pub fn set_switchboard(&mut self, id: OwnedRoomId, switchboard: SwitchboardHandle) -> Option<SwitchboardHandle> {
        self.inner.switchboards.insert(id, switchboard)
    }

    pub fn remove_switchboard(&mut self, id: &OwnedRoomId) -> Option<(OwnedRoomId, SwitchboardHandle)> {
        self.inner.switchboards.remove(id)
    }

    pub fn get_user(&self) -> Result<RwLockReadGuard<MsnUser>, ClientStoreError> {
        let out = self.inner.user.read().map_err(|e| ClientStoreError::PoisonnedLockError {name: "User".into(), source: anyhow!(e.to_string())})?;
        Ok(out)
    }

    pub fn get_user_clone(&self) -> Result<MsnUser, ClientStoreError> {
        Ok(self.get_user()?.clone())
    }

    pub fn get_user_mut(&mut self) -> Result<RwLockWriteGuard<MsnUser>, ClientStoreError> {
        Ok(self.inner.user.write().map_err(|e| ClientStoreError::PoisonnedLockError {name: "User".into(), source: anyhow!(e.to_string())})?)
    }

    pub fn add_oim(&mut self, oim: OIM) {
        self.inner.soap_holder.oims.insert(oim.message_id.clone(), oim);
    }

    pub fn get_oims(&mut self) -> &DashMap<String, OIM> {
        &self.inner.soap_holder.oims
    }

    pub fn get_contact_list(&self) -> &Mutex<ContactList> {
        &self.inner.contact_list
    }

    pub fn get_contact_holder_mut(&mut self) -> LockResult<MutexGuard<'_, Vec<Contact>>> {
       self.inner.soap_holder.contacts.lock()
    }

    pub fn get_member_holder_mut(&mut self) -> LockResult<MutexGuard<'_, VecDeque<BaseMember>>> {
        self.inner.soap_holder.memberships.lock()

    }

    pub fn remove_oim(&mut self, message_id: &str) -> Option<(String, OIM)> {
        self.inner.soap_holder.oims.remove(message_id)
    }

    pub fn get_ticket_token(&self) -> &TicketToken {
        &self.inner.ticket_token
    }

    pub fn get_matrix_client(&self) -> Client {
        self.inner.matrix_client.clone()
    }

    
    pub fn get_sync_service(&self) -> &SyncService {
        &self.inner.sync_service
    }
    
    //pub fn get_msn_client_handle(&self) -> MSNClientHandle {
    //    MSNClientHandle::new(self.clone())
    //}
}


// pub struct MSNClientHandle {
//     client_data: ClientData,
// }
//
// impl MSNClientHandle {
//     pub fn new(client_data: ClientData) -> MSNClientHandle {
//         MSNClientHandle { client_data }
//     }
//
//     pub async fn send_notification(&self, payload: NotificationPayload) -> Result<(), SendError<NotificationServerCommand>> {
//         self.client_data.inner.notification_sender.clone().send(
//             NotificationServerCommand::NOT(
//                 NotServer {
//                     payload,
//                 }
//             )
//         )?
//     }
// }

#[derive(Clone, Default)]
pub struct ClientStoreFacade {
    data: Arc<DashMap<String, ClientData>>
}

impl ClientStoreFacade {

    pub fn get_client_data(&self, key: &str) -> Option<ClientData> {
        match self.data.get(key) {
            None => {
                None
            }
            Some(found) => {
                Some(found.value().clone())
            }
        }
    }

    pub fn insert_client_data(&self, key: String, client_data: ClientData) {
        self.data.insert(key, client_data);
    }

    pub fn remove_client_data(&self, key: &str) -> Option<(String, ClientData)> {
        self.data.remove(key)
    }

}