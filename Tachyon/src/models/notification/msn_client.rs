use std::{
    sync::{
        Arc,
        atomic::{AtomicI16, Ordering}, RwLock, RwLockWriteGuard,
    },
};
use std::collections::HashMap;
use std::sync::Mutex;
use anyhow::anyhow;

use chashmap::CHashMap;
use log::{debug, error};
use matrix_sdk::Client;
use matrix_sdk::ruma::__private_macros::room_id;
use matrix_sdk::ruma::owned_room_id;
use tokio::sync::mpsc::UnboundedSender;

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
use crate::models::notification::events::notification_event::{NotificationEvent, NotificationEventFactory};
use crate::models::switchboard::switchboard::Switchboard;
use crate::models::tachyon_error::MatrixError;
use crate::utils::identifiers;
use crate::utils::identifiers::matrix_room_id_to_annoying_matrix_room_id;
use crate::utils::string::encode_base64;

use super::{adl_payload::ADLPayload, error::MSNPErrorCode};

#[derive(Clone, Debug)]
pub struct MSNClient {
    pub(crate) inner: Arc<MSNClientInner>,
}

#[derive(Debug)]
pub(crate) struct MSNClientInner {
    user: RwLock<MSNUser>,
    user_display_picture_dedup: Mutex<String>,
    user_display_name_dedup: Mutex<String>,
    msnp_version: AtomicI16,
    switchboards: SwitchboardRepository,
    matrix_client: Client,
    contact_list: CHashMap<RoleId, Vec<PartialMSNUser>>,
    sender: UnboundedSender<NotificationEvent>
}

impl Drop for MSNClientInner {
    fn drop(&mut self) {
        log::info!("MSN Client Inner Dropped!");
    }
}

impl MSNClient {
    pub fn new(matrix_client: Client, user: MSNUser, msnp_version: i16, notification_sender: UnboundedSender<NotificationEvent>) -> Self {
        let inner = Arc::new(MSNClientInner {
            user: RwLock::new(user),
            user_display_picture_dedup: Mutex::new(String::new()),
            user_display_name_dedup: Mutex::new(String::new()),
            msnp_version: AtomicI16::new(msnp_version),
            switchboards: SwitchboardRepository::new(),
            matrix_client,
            contact_list: CHashMap::new(),
            sender: notification_sender,
        });

        return MSNClient { inner };
    }

    pub fn get_user(&self) -> MSNUser {
        return self.inner.user.read().expect("user to be present in the msn_client").clone();
    }

    pub fn get_user_msn_addr(&self) -> String {
        return self.get_user().get_msn_addr();
    }

    pub fn get_user_mut(&mut self) -> RwLockWriteGuard<MSNUser> {
        return self.inner.user.write().expect("mutable user to be present in the msn_client");
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

    pub fn get_or_init_switchboard(&self, sb_id: String, inviter: MSNUser) -> Switchboard {
        let matrix_client = &self.inner.matrix_client;
        if let Some(switchboard) = self.get_switchboards().find(&sb_id) {
            switchboard
        } else {
            //sb not initialized yet
            let switchboard = Switchboard::new(matrix_client.clone(), matrix_room_id_to_annoying_matrix_room_id(&sb_id), matrix_client.user_id().expect("UserId to be present").to_owned());
            self.get_switchboards().add(sb_id.clone(), switchboard.clone());

            //send RNG command
            let session_id = identifiers::get_sb_session_id();
            let ticket = encode_base64(format!("{target_room_id};{token};{target_matrix_id}", target_room_id = &sb_id, token = &matrix_client.access_token().unwrap(), target_matrix_id = inviter.get_matrix_id().to_string()));
            self.inner.sender.send(NotificationEventFactory::get_switchboard_init(inviter, session_id, ticket)).expect("Sending event to not fail");
            switchboard
        }
    }

    pub async fn get_mpop_endpoints(&self) -> Result<Vec<MPOPEndpoint>, MatrixError> {
        let client = &self.inner.matrix_client;
        let devices = client.devices().await?.devices;

        let mut endpoints: Vec<MPOPEndpoint> = Vec::new();

        let this_device_id = client.device_id().expect("The client to be logged-in when fetching MPOP endpoints");

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

    pub fn add_to_contact_list(&mut self, adl_payload: &ADLPayload) {
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

    pub fn remove_from_contact_list(&mut self, adl_payload: &ADLPayload) {
        for domain in &adl_payload.domains {
            let email_domain = &domain.domain;

            for contact in &domain.contacts {
                let contact_list_types = contact.get_roles();

                let msn_addr = format!("{}@{}", &contact.email_part, &email_domain);
                let partial_user = PartialMSNUser::new(msn_addr);

                for contact_list_type in contact_list_types {
                    if let Some(mut found) = self.inner.contact_list.get_mut(&contact_list_type) {
                        let maybe_found_index = found.iter().position(|n| n == &partial_user);
                        if let Some(index) = maybe_found_index {
                            let _osef = found.remove(index);
                        }
                    }
                }
            }
        }

        debug!("Debug contact_list parsed: {:?}", self.inner.contact_list);
    }

    pub async fn get_contacts(&self, fetch_presence: bool) -> Vec<MSNUser> {
        let repo = MSNUserRepository::new(self.inner.matrix_client.clone());

        let forward_contacts = self.inner.contact_list.get(&RoleId::Allow);

        let mut out = Vec::new();

        if forward_contacts.is_none() {
            return out;
        }

        let partial_contacts = forward_contacts.unwrap();
        for i in 0..partial_contacts.len() {
            let current = partial_contacts.get(i).expect("contact list array to be in bounds");

            let msn_user = repo.get_msnuser_from_userid(&current.get_matrix_id(), fetch_presence).await;
            if let Ok(msn_user) = msn_user {
                out.push(msn_user);
            } else if let Err(err) = msn_user{
                error!("{}", err);
            }
        }

        return out;
    }

    pub fn on_user_disconnected(&self, user: MSNUser) {
        self.inner.sender.send(NotificationEventFactory::get_disconnect(user));
    }
    pub fn on_user_presence_changed(&self, user: MSNUser) {
        self.inner.sender.send(NotificationEventFactory::get_presence(user));
    }
    pub fn on_notify_ab_update(&self) {
        self.inner.sender.send(NotificationEventFactory::get_ab_updated(self.get_user()));
    }
}
