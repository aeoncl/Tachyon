use matrix_sdk::ruma::{OwnedUserId};

use crate::{generated::payloads::PresenceStatus};

use super::{uuid::{UUID, PUID}, capabilities::{ClientCapabilitiesFactory, ClientCapabilities}, errors::Errors, msn_object::MSNObject, owned_user_id_traits::{FromMsnAddr, ToMsnAddr}};

#[derive(Clone, Debug)]
pub struct PartialMSNUser {
    msn_addr: String,
    matrix_id : OwnedUserId,
}

impl PartialMSNUser {
    pub fn new(msn_addr: String) -> PartialMSNUser {
        let matrix_id = OwnedUserId::from_msn_addr(&msn_addr);
        return Self::from_wlmatrix_identifiers(matrix_id, msn_addr);
       
    }

    pub fn from_matrix_id(matrix_id: OwnedUserId) -> PartialMSNUser {
        let msn_addr = matrix_id.to_msn_addr();
        return Self::from_wlmatrix_identifiers(matrix_id, msn_addr);

    }

    pub(self) fn from_wlmatrix_identifiers(matrix_id: OwnedUserId, msn_addr: String) -> Self {
        return PartialMSNUser{msn_addr, matrix_id};
    }

    pub fn get_uuid(&self) -> UUID {
        return UUID::from_string(&self.matrix_id.to_string());
    }

    pub fn get_puid(&self) -> PUID {
        return self.get_uuid().get_puid();
    }

    pub fn get_msn_addr(&self) -> String {
        return self.msn_addr.clone();
    }

    pub fn get_matrix_id(&self) -> OwnedUserId {
        return self.matrix_id.clone();
    }

}

impl Into<PUID> for PartialMSNUser {
    fn into(self) -> PUID {
        return self.get_puid();
    }
}

impl Into<UUID> for PartialMSNUser {
    fn into(self) -> UUID {
        return self.get_uuid();
    }
}

impl From<OwnedUserId> for PartialMSNUser {
    fn from(matrix_id: OwnedUserId) -> Self {
        PartialMSNUser::from_matrix_id(matrix_id)
    }
}

impl Into<OwnedUserId> for PartialMSNUser {
    fn into(self) -> OwnedUserId {
        self.matrix_id
    }
}

impl From<MSNUser> for PartialMSNUser {

    fn from(value: MSNUser) -> Self {
        value.get_partial_msnuser()
    }
}


#[derive(Clone, Debug)]
pub struct MSNUser {
    partial_user: PartialMSNUser,
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

    pub fn new(msn_addr: String) -> Self {
        let matrix_id = OwnedUserId::from_msn_addr(&msn_addr);
        return Self::from_partial_user(PartialMSNUser::from_wlmatrix_identifiers(matrix_id, msn_addr));
    }

    pub fn from_matrix_id(matrix_id: OwnedUserId) -> Self {
        let msn_addr = matrix_id.to_msn_addr();
        return Self::from_partial_user(PartialMSNUser::from_wlmatrix_identifiers(matrix_id, msn_addr));
    }

    pub fn from_partial_user(partial_user: PartialMSNUser) -> Self {
        let msn_addr = partial_user.get_msn_addr();
        let endpoint_guid = UUID::from_string(&msn_addr).to_string().to_uppercase();
        return MSNUser{partial_user, 
            display_name: msn_addr.clone(),
            capabilities: ClientCapabilitiesFactory::get_default_capabilities(), 
            status: PresenceStatus::FLN, 
            endpoint_guid: endpoint_guid,
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
            let mut out : MSNUser = MSNUser::new(trimmed_msn_addr);
            out.set_endpoint_guid(trimmed_endpoint_guid.to_string());
            return Ok(out);
       }
       return Err(Errors::PayloadDeserializeError);
    }

    pub fn get_msn_addr(&self) -> String {
        return self.partial_user.get_msn_addr()
    }

    pub fn get_matrix_id(&self) -> OwnedUserId {
        return self.partial_user.get_matrix_id();
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
       return self.partial_user.get_uuid();
    }

    pub fn get_puid(&self) -> PUID {
        return self.partial_user.get_puid();
    }

    pub fn set_display_picture(&mut self, display_picture: Option<MSNObject>) {
        self.display_picture = display_picture;
    }

    pub fn get_display_picture(&self) -> Option<MSNObject> {
        return self.display_picture.clone();
    }

    pub fn get_partial_msnuser(&self) -> PartialMSNUser {
        return self.partial_user.clone();
    }
    
    pub fn get_mpop_identifier(&self) -> String {
        return format!("{msn_addr};{{{endpoint_guid}}}", msn_addr = &self.partial_user.msn_addr, endpoint_guid = &self.endpoint_guid);
    }
}