use std::{
    mem,
    path::Path,
    sync::{
        atomic::{AtomicI16, Ordering, AtomicBool},
        Arc, Mutex, RwLock, RwLockWriteGuard,
    },
};

use matrix_sdk::{
    ruma::{device_id, DeviceId, OwnedUserId, UserId},
    Client, Session,
};
use tokio::sync::{broadcast::{self, Receiver, Sender}, oneshot};

use crate::{
    models::{msn_user::MSNUser, uuid::UUID, capabilities::{ClientCapabilities, ClientCapabilitiesFactory}},
    repositories::switchboard_repository::SwitchboardRepository,
    utils::{identifiers::{get_matrix_device_id, msn_addr_to_matrix_user_id}, matrix_sync_helpers::start_matrix_loop}, MATRIX_CLIENT_LOCATOR, generated::payloads::{MPOPEndpoint, EndpointData, PrivateEndpointData, ClientType, PresenceStatus},
};

use super::error::{MsnpError, MsnpErrorCode};

#[derive(Clone, Debug)]

pub struct MSNClient {
    matrix_client: Option<Client>,
    pub(crate) inner: Arc<MSNClientInner>,
}

#[derive(Debug)]

pub(crate) struct MSNClientInner {
    user: RwLock<MSNUser>,
    msnp_version: AtomicI16,
    switchboards: SwitchboardRepository,
    notification_sender: Sender<String>,
    notification_receiver: Mutex<Option<Receiver<String>>>,
    stop_listen_sender: Mutex<Option<oneshot::Sender::<()>>>
}

impl Drop for MSNClientInner {
    fn drop(&mut self) {
        log::info!("MSN Client Inner Dropped!");
    }
}

impl MSNClient {
    pub fn new(user: MSNUser, msnp_version: i16) -> Self {
        let (notification_sender, notification_receiver) = broadcast::channel::<String>(30);


        let inner = Arc::new(MSNClientInner {
            user: RwLock::new(user),
            msnp_version: AtomicI16::new(msnp_version),
            switchboards: SwitchboardRepository::new(),
            notification_sender,
            notification_receiver: Mutex::new(Some(notification_receiver)),
            stop_listen_sender: Mutex::new(None)
        });

        return MSNClient {
            inner,
            matrix_client: None,
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

    pub fn get_receiver(&mut self) -> Receiver<String> {
        let mut lock = self.inner.notification_receiver.lock().unwrap();
        if lock.is_none() {
            return self.inner.notification_sender.subscribe();
        } else {
            let receiver = mem::replace(&mut *lock, None).unwrap();
            return receiver;
        }
    }

    pub fn update_device_name(device_name: String) -> Result<(), ()> {
        // if let Some(matrix_client) = MATRIX_CLIENT_LOCATOR.get() {
        // }

        Ok(())
    }

    pub async fn get_mpop_endpoints(&self) -> Result<Vec<MPOPEndpoint>, MsnpErrorCode> {
        let client = self.matrix_client.as_ref().ok_or(MsnpErrorCode::InternalServerError)?;
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

    pub async fn login(&mut self, token: String) -> Result<(), MsnpErrorCode> {
        let matrix_id = msn_addr_to_matrix_user_id(&self.get_user_msn_addr());

        let device_id = get_matrix_device_id();
        let device_id = device_id!(device_id.as_str()).to_owned();

        let path = Path::new("c:\\temp");
        match Client::builder()
            .disable_ssl_verification()
            .server_name(matrix_id.server_name())
            .sled_store(path, None)
            .build()
            .await
        {
            Ok(client) => {
                if let Err(err) = client
                    .restore_session(Session {
                        access_token: token,
                        refresh_token: None,
                        user_id: matrix_id,
                        device_id: device_id,
                    })
                    .await
                {
                    return Err(MsnpErrorCode::AuthFail);
                }

                if let Err(_check_connection_status) = client.whoami().await {
                    return Err(MsnpErrorCode::AuthFail);
                }

                self.matrix_client = Some(client.clone());
                MATRIX_CLIENT_LOCATOR.set(client);
                return Ok(());
            }
            Err(_err) => {
                log::error!("An error has occured building the client: {}", _err);
                return Err(MsnpErrorCode::AuthFail);
            }
        }
    }

    pub async fn listen(&self, todoRemoveThis: Sender<String>) -> Result<(),()> {
        let kill_sender = start_matrix_loop(self.matrix_client.as_ref().ok_or(())?.clone(), self.get_user(), todoRemoveThis).await;
        self.inner.stop_listen_sender.lock().unwrap().insert(kill_sender);
        Ok(())
    }
}

impl Drop for MSNClient {
    fn drop(&mut self) {
        let mut lock = self.inner.stop_listen_sender.lock().unwrap();
        if lock.is_some() { 
           let sender = lock.take().unwrap();
           sender.send(());
        }

    }
}
