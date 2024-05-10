use anyhow::anyhow;

use crate::{msnp::error::PayloadError, shared::models::uuid::{Puid, Uuid}};
use crate::msnp::notification::models::endpoint_guid::EndpointGuid;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::endpoint_id::EndpointId;

use super::{capabilities::ClientCapabilities, msn_object::MsnObject, presence_status::PresenceStatus};


#[derive(Clone, Debug)]
pub struct MsnUser {
    pub endpoint_id: EndpointId,
    pub uuid: Uuid,
    pub capabilities: ClientCapabilities,
    pub status: PresenceStatus,
    pub display_name: String,
    pub psm: String,
    pub display_picture: Option<MsnObject>
}

impl MsnUser {

    pub fn new(endpoint_id: EndpointId) -> Self {
        let uuid = Uuid::from_seed(&endpoint_id.email_addr.0);
        MsnUser {
            endpoint_id,
            uuid,
            display_name: String::default(),
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
        if !self.display_name.is_empty() {
            &self.display_name
        } else {
            &self.endpoint_id.email_addr.0
        }
    }

    pub fn get_email_address(&self) -> &EmailAddress {
        &self.endpoint_id.email_addr
    }
    
}