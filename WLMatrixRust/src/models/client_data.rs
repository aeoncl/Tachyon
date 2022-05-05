use crate::{generated::payloads::PresenceStatus, repositories::{switchboard_repository::SwitchboardRepository, repository::Repository}};

pub struct ClientData {
    pub msn_login: String,
    pub msnp_version: i16,
    pub msn_machine_guid: String,
    pub presence_status: PresenceStatus,
    pub switchboards : SwitchboardRepository
}

impl ClientData {
    pub fn new(msn_login: String, msnp_version: i16, msn_machine_guid: String, presence_status: PresenceStatus) -> ClientData {
        return ClientData{ msn_login, msnp_version, msn_machine_guid, presence_status, switchboards: SwitchboardRepository::new()};
    }
}

