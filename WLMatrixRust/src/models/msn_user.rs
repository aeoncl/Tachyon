use matrix_sdk::ruma::api::client::user_directory::search_users::v3::User;

use crate::{utils::identifiers::{msn_addr_to_matrix_id, matrix_id_to_msn_addr}, generated::payloads::PresenceStatus};

pub struct MSNUser {

    pub msn_addr: String,
    pub matrix_id : String,
    pub capabilities: String,
    pub status: PresenceStatus,
}

impl MSNUser {

    pub fn new(msn_addr: String) -> MSNUser {
        return MSNUser{msn_addr: msn_addr.clone(), matrix_id: msn_addr_to_matrix_id(&msn_addr), capabilities: String::new(), status: PresenceStatus::FLN };
    }

    pub fn from_matrix_id(matrix_id: String) -> MSNUser {
        return MSNUser{msn_addr: matrix_id_to_msn_addr(&matrix_id), matrix_id: matrix_id.clone(), capabilities: String::new(), status: PresenceStatus::FLN};
    }


}