pub struct ClientData {
    pub msn_login: String,
    pub msnp_version: i16,
    pub msn_machine_guid: String
}

impl ClientData {
    pub fn new(msn_login: String, msnp_version: i16, msn_machine_guid: String) -> ClientData {
        return ClientData{ msn_login, msnp_version, msn_machine_guid};
    }
}

