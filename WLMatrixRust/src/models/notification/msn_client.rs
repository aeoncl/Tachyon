use std::{
    mem,
    sync::{
        atomic::{AtomicI16, Ordering},
        Arc, Mutex, RwLock, RwLockWriteGuard,
    }
};

use matrix_sdk::{
    Client
};

use crate::{
    models::{msn_user::MSNUser, uuid::UUID, capabilities::{ClientCapabilitiesFactory}},
    repositories::switchboard_repository::SwitchboardRepository,
    generated::payloads::{MPOPEndpoint, EndpointData, PrivateEndpointData, ClientType, PresenceStatus},
};

use super::{error::{MsnpErrorCode}};

#[derive(Clone, Debug)]
pub struct MSNClient {
    pub(crate) inner: Arc<MSNClientInner>,
}

#[derive(Debug)]
pub(crate) struct MSNClientInner {
    user: RwLock<MSNUser>,
    msnp_version: AtomicI16,
    switchboards: SwitchboardRepository,
    matrix_client: Client
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
            matrix_client
        });

        return MSNClient {
            inner
        };
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

        let this_device_id = client.device_id().ok_or(MsnpErrorCode::InternalServerError)?;

        for device in devices {
            if device.device_id != this_device_id {
                let machine_guid = format!("{{{}}}", UUID::from_string(&device.device_id.to_string()));
                let endpoint_name = device.display_name.unwrap_or(device.device_id.to_string());

                let endpoint = EndpointData::new(Some(machine_guid.to_string()), ClientCapabilitiesFactory::get_default_capabilities());
                let private_endpoint  = PrivateEndpointData::new(Some(machine_guid.to_string()), endpoint_name, false, ClientType::Computer, PresenceStatus::NLN );
                
                endpoints.push(MPOPEndpoint::new(endpoint, private_endpoint));
            }
        }
        Ok(endpoints)
    }
}
