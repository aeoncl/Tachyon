use matrix_sdk::ruma::api::client::user_directory::search_users::v3::User;

use crate::{utils::identifiers::{msn_addr_to_matrix_id, matrix_id_to_msn_addr}, generated::payloads::PresenceStatus};

use super::{uuid::UUID, capabilities::ClientCapabilitiesFactory, errors::Errors};

#[derive(Clone, Debug)]

pub struct MSNUser {

    pub msn_addr: String,
    pub matrix_id : String,
    pub capabilities: String,
    pub status: PresenceStatus,
    pub endpoint_guid: String,
}

impl MSNUser {

    pub fn new(msn_addr: String) -> MSNUser {

        let endpoint_guid = UUID::from_string(&msn_addr).to_string().to_uppercase();
        return MSNUser{msn_addr: msn_addr.clone(), 
            matrix_id: msn_addr_to_matrix_id(&msn_addr), 
           // capabilities: ClientCapabilitiesFactory::get_default_capabilities().to_string(), 
            capabilities: String::from("2788999228:48"), 
            status: PresenceStatus::FLN, 
            endpoint_guid: endpoint_guid };
    }

    pub fn from_matrix_id(matrix_id: String) -> MSNUser {
        let msn_addr = matrix_id_to_msn_addr(&matrix_id);
        let endpoint_guid = UUID::from_string(&msn_addr).to_string().to_uppercase();
        return MSNUser{msn_addr: msn_addr, 
            matrix_id: matrix_id.clone(), 
            //capabilities: ClientCapabilitiesFactory::get_default_capabilities().to_string(), 
            capabilities: String::from("2788999228:48"),
            status: PresenceStatus::FLN, endpoint_guid: 
            endpoint_guid };
    }

    /**
     * Parses from a msn_addr;{endpoint_guid} string
     */
    pub fn from_mpop_addr_string(mpop_string: String) -> Result<MSNUser, Errors> {
       if let Some((msn_addr, endpoint_guid)) = mpop_string.split_once(";") {
           let  trimmed_msn_addr = msn_addr.trim().to_string();
            let trimmed_endpoint_guid = endpoint_guid.trim().strip_prefix("{").ok_or(Errors::PayloadDeserializeError)?.strip_suffix("}").ok_or(Errors::PayloadDeserializeError)?;
            return Ok(MSNUser{ msn_addr: trimmed_msn_addr.clone(), matrix_id: msn_addr_to_matrix_id(&trimmed_msn_addr), capabilities: String::from("2788999228:48"), status: PresenceStatus::FLN, endpoint_guid: trimmed_endpoint_guid.to_string() });
       }
       return Err(Errors::PayloadDeserializeError);
    }


}