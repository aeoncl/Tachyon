use std::{
    sync::{
        Arc,
        atomic::{AtomicI16, Ordering}, RwLock, RwLockWriteGuard,
    },
};

use chashmap::CHashMap;
use log::debug;
use matrix_sdk::Client;

use crate::{
    generated::{
        msnab_datatypes::types::RoleId,
        payloads::{ClientType, EndpointData, MPOPEndpoint, PresenceStatus, PrivateEndpointData},
    },
    models::{
        capabilities::ClientCapabilitiesFactory,
        msn_user::{MSNUser, PartialMSNUser},
        uuid::UUID,
    },
    repositories::{msn_user_repository::MSNUserRepository, switchboard_repository::SwitchboardRepository},
};

use super::{adl_payload::ADLPayload, error::MsnpErrorCode};

#[derive(Clone, Debug)]
pub struct MSNClient {
    pub(crate) inner: Arc<MSNClientInner>,
}

#[derive(Debug)]
pub(crate) struct MSNClientInner {
    user: RwLock<MSNUser>,
    msnp_version: AtomicI16,
    switchboards: SwitchboardRepository,
    matrix_client: Client,
    contact_list: CHashMap<RoleId, Vec<PartialMSNUser>>,
}

impl Drop for MSNClientInner {
    fn drop(&mut self) {
        log::info!("MSN Client Inner Dropped!");
    }
}

impl MSNClient {
    pub fn new(matrix_client: Client, user: MSNUser, msnp_version: i16) -> Self {
        let inner = Arc::new(MSNClientInner {
            user: RwLock::new(user),
            msnp_version: AtomicI16::new(msnp_version),
            switchboards: SwitchboardRepository::new(),
            matrix_client,
            contact_list: CHashMap::new(),
        });

        return MSNClient { inner };
    }

    pub fn get_user(&self) -> MSNUser {
        return self.inner.user.read().unwrap().clone();
    }

    pub fn get_user_msn_addr(&self) -> String {
        return self.inner.user.read().unwrap().get_msn_addr();
    }

    pub fn get_user_mut(&mut self) -> RwLockWriteGuard<MSNUser> {
        return self.inner.user.write().unwrap();
    }

    pub fn set_msnp_version(&mut self, msnp_version: i16) {
        self.inner
            .msnp_version
            .store(msnp_version, Ordering::SeqCst);
    }

    pub fn get_msnp_version(&self) -> i16 {
        return self.inner.msnp_version.load(Ordering::SeqCst);
    }

    pub fn get_switchboards(&self) -> &SwitchboardRepository {
        return &self.inner.switchboards;
    }

    pub fn update_device_name(device_name: String) -> Result<(), ()> {
        // if let Some(matrix_client) = MATRIX_CLIENT_LOCATOR.get() {
        // }

        Ok(())
    }

    pub async fn get_mpop_endpoints(&self) -> Result<Vec<MPOPEndpoint>, MsnpErrorCode> {
        let client = &self.inner.matrix_client;
        let devices = client.devices().await?.devices;

        let mut endpoints: Vec<MPOPEndpoint> = Vec::new();

        let this_device_id = client
            .device_id()
            .ok_or(MsnpErrorCode::InternalServerError)?;

        for device in devices {
            if device.device_id != this_device_id {
                let machine_guid =
                    format!("{{{}}}", UUID::from_string(&device.device_id.as_str()));
                let endpoint_name = device.display_name.unwrap_or(device.device_id.to_string());

                let endpoint = EndpointData::new(
                    Some(machine_guid.to_string()),
                    ClientCapabilitiesFactory::get_default_capabilities(),
                );
                let private_endpoint = PrivateEndpointData::new(
                    Some(machine_guid.to_string()),
                    endpoint_name,
                    false,
                    ClientType::Computer,
                    PresenceStatus::NLN,
                );

                endpoints.push(MPOPEndpoint::new(endpoint, private_endpoint));
            }
        }
        Ok(endpoints)
    }

    pub fn init_contact_list(&mut self, adl_payload: &ADLPayload) {
        for domain in &adl_payload.domains {
            let email_domain = &domain.domain;

            for contact in &domain.contacts {
                let contact_list_types = contact.get_roles();

                let msn_addr = format!("{}@{}", &contact.email_part, &email_domain);
                let partial_user = PartialMSNUser::new(msn_addr);

                for contact_list_type in contact_list_types {
                    if let Some(mut found) = self.inner.contact_list.get_mut(&contact_list_type) {
                        found.push(partial_user.clone());
                    } else {
                        let mut vec = Vec::new();
                        vec.push(partial_user.clone());
                        self.inner.contact_list.insert(contact_list_type, vec);
                    }
                }
            }
        }

        debug!("Debug contact_list parsed: {:?}", self.inner.contact_list);
    }

    pub async fn get_contacts(&self, fetch_presence: bool) -> Vec<MSNUser> {
        let repo = MSNUserRepository::new(self.inner.matrix_client.clone());
        let forward_contacts = self.inner.contact_list.get(&RoleId::Forward);

        let mut out = Vec::new();


        if forward_contacts.is_none() {
            return out;
        }


        let partial_contacts = forward_contacts.as_ref().unwrap();
        for i in 0..partial_contacts.len() {
            let current = partial_contacts.get(i).expect("contact list array to be in bounds");

            let msn_user = repo.get_msnuser_from_userid(&current.get_matrix_id(), fetch_presence).await;
            if let Ok(msn_user) = msn_user {
                out.push(msn_user);
            }
        }

        return out;
    }
}
