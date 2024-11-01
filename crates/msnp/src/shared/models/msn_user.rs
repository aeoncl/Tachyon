use anyhow::anyhow;

use crate::{msnp::error::PayloadError, shared::models::uuid::{Puid, Uuid}};
use crate::msnp::notification::models::endpoint_guid::EndpointGuid;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::models::network_id::NetworkId;
use crate::shared::models::network_id_email::NetworkIdEmail;

use super::{capabilities::ClientCapabilities, msn_object::MsnObject, presence_status::PresenceStatus};


#[derive(Clone, Debug)]
pub struct MsnUser {
    pub endpoint_id: EndpointId,
    pub network_id: NetworkId,
    pub uuid: Uuid,
    pub capabilities: ClientCapabilities,
    pub status: PresenceStatus,
    pub display_name: Option<String>,
    pub psm: String,
    pub display_picture: Option<MsnObject>
}

impl MsnUser {

    pub fn new(endpoint_id: EndpointId) -> Self {
        let uuid = Uuid::from_seed(&endpoint_id.email_addr.0);
        MsnUser {
            endpoint_id,
            network_id: NetworkId::WindowsLive,
            uuid,
            display_name: None,
            capabilities: ClientCapabilities::default(), 
            status: PresenceStatus::default(), 
            psm: String::default(),
            display_picture: None
        }
    }

    pub fn with_email_addr(email_addr: EmailAddress) -> Self {
        Self::new(EndpointId::from_email_addr(email_addr))
    }

    pub fn without_endpoint_guid(email_addr: EmailAddress) -> Self {
        Self::new(EndpointId::new(email_addr, None))
    }

    pub fn compute_display_name(&self) -> &str {
        if self.display_name.is_some() {
            self.display_name.as_ref().expect("yes")
        } else {
            &self.endpoint_id.email_addr.0
        }
    }

    pub fn get_email_address(&self) -> &EmailAddress {
        &self.endpoint_id.email_addr
    }

    pub fn get_network_id_email(&self) -> NetworkIdEmail {
        NetworkIdEmail::new(self.network_id.clone(), self.get_email_address().clone())
    }
    
}