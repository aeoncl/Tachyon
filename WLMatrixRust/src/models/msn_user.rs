use matrix_sdk::ruma::api::client::user_directory::search_users::v3::User;

use crate::{utils::identifiers::{msn_addr_to_matrix_id, matrix_id_to_msn_addr}, generated::payloads::PresenceStatus};

use super::{uuid::UUID, capabilities::ClientCapabilitiesFactory};

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
            capabilities: ClientCapabilitiesFactory::get_default_capabilities().to_string(), 
            status: PresenceStatus::FLN, 
            endpoint_guid: endpoint_guid };
    }

    pub fn from_matrix_id(matrix_id: String) -> MSNUser {
        let msn_addr = matrix_id_to_msn_addr(&matrix_id);
        let endpoint_guid = UUID::from_string(&msn_addr).to_string().to_uppercase();
        return MSNUser{msn_addr: msn_addr, 
            matrix_id: matrix_id.clone(), 
            capabilities: ClientCapabilitiesFactory::get_default_capabilities().to_string(), 
            status: PresenceStatus::FLN, endpoint_guid: 
            endpoint_guid };
    }


}