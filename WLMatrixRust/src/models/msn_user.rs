use crate::{utils::identifiers::{msn_addr_to_matrix_id, matrix_id_to_msn_addr}, generated::payloads::PresenceStatus};

use super::{uuid::{UUID, PUID}, capabilities::{ClientCapabilitiesFactory, ClientCapabilities}, errors::Errors, msn_object::MSNObject};

#[derive(Clone, Debug)]

pub struct MSNUser {

    msn_addr: String,
    matrix_id : String,
    capabilities: ClientCapabilities,
    status: PresenceStatus,
    endpoint_guid: String,
    display_name: String,
    psm: String,
    display_picture: Option<MSNObject>
}

impl MSNUser {

    pub fn default() -> MSNUser {
        return MSNUser::new(String::from("johndoe@default.com"));
    }

    pub fn new(msn_addr: String) -> MSNUser {

        let endpoint_guid = UUID::from_string(&msn_addr).to_string().to_uppercase();
        return MSNUser{msn_addr: msn_addr.clone(), 
            display_name: msn_addr.clone(),
            matrix_id: msn_addr_to_matrix_id(&msn_addr), 
            capabilities: ClientCapabilitiesFactory::get_default_capabilities(), 
            //capabilities: String::from("2788999228:48"), 
            status: PresenceStatus::FLN, 
            endpoint_guid: endpoint_guid,
            psm: String::new(),
            display_picture: None };
    }

    pub fn from_matrix_id(matrix_id: String) -> MSNUser {
        let msn_addr = matrix_id_to_msn_addr(&matrix_id);
        let endpoint_guid = UUID::from_string(&msn_addr).to_string().to_uppercase();
        return MSNUser{msn_addr: msn_addr.clone(), 
            matrix_id: matrix_id.clone(), 
            display_name: msn_addr.clone(),
            capabilities: ClientCapabilitiesFactory::get_default_capabilities(), 
            //capabilities: String::from("2788999228:48"),
            status: PresenceStatus::FLN, endpoint_guid: 
            endpoint_guid,
            psm: String::new(),
            display_picture: None };
    }

    /**
     * Parses from a msn_addr;{endpoint_guid} string
     */
    pub fn from_mpop_addr_string(mpop_string: String) -> Result<MSNUser, Errors> {
       if let Some((msn_addr, endpoint_guid)) = mpop_string.split_once(";") {
           let  trimmed_msn_addr = msn_addr.trim().to_string();
            let trimmed_endpoint_guid = endpoint_guid.trim().strip_prefix("{").ok_or(Errors::PayloadDeserializeError)?.strip_suffix("}").ok_or(Errors::PayloadDeserializeError)?;
            let capab = ClientCapabilitiesFactory::get_default_capabilities();
            return Ok(MSNUser{ msn_addr: trimmed_msn_addr.clone(),display_name:trimmed_msn_addr.clone(), matrix_id: msn_addr_to_matrix_id(&trimmed_msn_addr), capabilities: capab, status: PresenceStatus::FLN, endpoint_guid: trimmed_endpoint_guid.to_string(), psm: String::new(), display_picture: None });
       }
       return Err(Errors::PayloadDeserializeError);
    }

    pub fn set_msn_addr(&mut self, msn_addr: String) {
        self.msn_addr = msn_addr;
        self.matrix_id = msn_addr_to_matrix_id(&self.msn_addr);
    }

    pub fn get_msn_addr(&self) -> String {
        return self.msn_addr.clone();
    }

    pub fn get_matrix_id(&self) -> String {
        return self.matrix_id.clone();
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

    pub fn get_uuid(&self) -> UUID {
       return UUID::from_string(&self.matrix_id);
    }

    pub fn get_puid(&self) -> PUID {
        return self.get_uuid().get_puid();
    }
    
}