use anyhow::anyhow;

use crate::{msnp::error::PayloadError, shared::models::uuid::{Puid, Uuid}};

use super::{capabilities::ClientCapabilities, msn_object::MSNObject, presence_status::PresenceStatus};


#[derive(Clone, Debug)]
pub struct MSNUser {
    email_addr: String,
    uuid: Uuid,
    capabilities: ClientCapabilities,
    status: PresenceStatus,
    endpoint_guid: String,
    display_name: String,
    psm: String,
    display_picture: Option<MSNObject>
}

impl MSNUser {

    pub fn default() -> Self {
        return MSNUser::new(String::from("johndoe@default.com"));
    }

    pub fn new(email_addr: String) -> Self {
        let uuid = Uuid::from_seed(&email_addr);
        let endpoint_guid = uuid.to_string().to_uppercase();

        MSNUser {
            email_addr: email_addr.clone(),
            uuid,
            display_name: email_addr,
            capabilities: ClientCapabilities::default(), 
            status: PresenceStatus::default(), 
            endpoint_guid,
            psm: String::default(),
            display_picture: None
        }

    }
    /**
     * Parses from a msn_addr;{endpoint_guid} string
     */
    pub fn from_mpop_addr_string(mpop_string: String) -> Result<MSNUser, PayloadError> {
       let (msn_addr, endpoint_guid) = mpop_string.split_once(";").ok_or(PayloadError::StringPayloadParsingError { payload: mpop_string.clone(), sauce: anyhow!("Couldn't split MPOP String on ;") })?;

        let  trimmed_msn_addr = msn_addr.trim().to_string();
            let trimmed_endpoint_guid = endpoint_guid.trim()
                .strip_prefix("{")
                .ok_or(PayloadError::StringPayloadParsingError { payload: endpoint_guid.to_string(), sauce: anyhow!("Couldn't strip {{ prefix from endpoint_guid") })?
                .strip_suffix("}")
                .ok_or(PayloadError::StringPayloadParsingError { payload: endpoint_guid.to_string(), sauce: anyhow!("Couldn't strip }} suffix from endpoint_guid") })?;

            let mut out : MSNUser = MSNUser::new(trimmed_msn_addr);
            out.set_endpoint_guid(trimmed_endpoint_guid.to_string());
            return Ok(out);

    }

    pub fn get_email_addr(&self) -> String {
        return self.email_addr.clone();
    }

    pub fn get_capabilities(&self) -> ClientCapabilities {
        return self.capabilities.clone();
    }

    pub fn set_status(&mut self, status: PresenceStatus) {
        self.status = status;
    }

    pub fn get_status(&self) -> PresenceStatus {
        return self.status.clone();
    }

    pub fn set_endpoint_guid(&mut self, endpoint_guid: String) {
        self.endpoint_guid = endpoint_guid;
    }

    pub fn get_endpoint_guid(&self) -> String {
        return self.endpoint_guid.clone();
    }

    pub fn set_display_name(&mut self, display_name: String) {
        self.display_name = urlencoding::encode(display_name.as_str()).into_owned();
    }

    pub fn get_display_name(&self) -> String {
        return self.display_name.clone();
    }

    pub fn set_psm(&mut self, psm: String){
        self.psm = psm;
    }

    pub fn get_psm(&self) -> String {
        return self.psm.clone();
    }

    pub fn get_uuid(&self) -> Uuid {
       return self.uuid.clone();
    }

    pub fn get_puid(&self) -> Puid {
        return self.uuid.get_puid();
    }

    pub fn set_display_picture(&mut self, display_picture: Option<MSNObject>) {
        self.display_picture = display_picture;
    }

    pub fn get_display_picture(&self) -> Option<MSNObject> {
        return self.display_picture.clone();
    }

    pub fn get_mpop_identifier(&self) -> String {
        return format!("{msn_addr};{{{endpoint_guid}}}", msn_addr = &self.email_addr, endpoint_guid = &self.endpoint_guid);
    }
    
}