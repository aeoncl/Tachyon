use std::collections::VecDeque;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use anyhow::anyhow;
use dashmap::DashMap;
use matrix_sdk::Client;
use matrix_sdk::ruma::OwnedRoomId;
use thiserror::__private::AsDynError;
use thiserror::Error;
use tokio::sync::mpsc;

use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::oim::OIM;
use msnp::shared::models::ticket_token::TicketToken;

#[derive(Clone)]
pub struct SwitchboardHandle {
    room_id: String,
    msnp_sender: mpsc::Sender<SwitchboardServerCommand>,
    p2p_sender: mpsc::Sender<String>
}

pub struct ClientDataInner {
    pub user: RwLock<MsnUser>,
    pub ticket_token: TicketToken,
    pub matrix_client: Client,
    pub contact_list: ContactList,
    pub soap_holder: SoapHolder,
    pub switchboards: DashMap<OwnedRoomId, SwitchboardHandle>,
}

#[derive(Default)]
pub struct SoapHolder {
    oims: DashMap<String, OIM>,
    contacts: VecDeque<String>,
    memberships: VecDeque<String>
}

#[derive(Clone)]
pub struct ClientData {
    inner: Arc<ClientDataInner>
}

#[derive(Error, Debug)]
pub enum ClientStoreError {
    #[error("Mutex lock was poisonned: {}", .name)]
    PoisonnedLockError{name: String, source: anyhow::Error}
}

impl ClientData {
    pub fn new(user: MsnUser, token: TicketToken, matrix_client: Client) -> ClientData {
        ClientData{ inner: Arc::new(ClientDataInner {
            user: RwLock::new(user),
            ticket_token: token,
            matrix_client,
            contact_list: Default::default(),
            soap_holder: Default::default(),
            switchboards: Default::default(),
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

    pub fn get_user_mut(&mut self) -> Result<RwLockWriteGuard<MsnUser>, ClientStoreError> {
        Ok(self.inner.user.write().map_err(|e| ClientStoreError::PoisonnedLockError {name: "User".into(), source: anyhow!(e.to_string())})?)
    }

    pub fn add_oim(&mut self, oim: OIM) {
        self.inner.soap_holder.oims.insert(oim.message_id.clone(), oim);
    }

    pub fn get_oims(&mut self) -> &DashMap<String, OIM> {
        &self.inner.soap_holder.oims
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
}


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