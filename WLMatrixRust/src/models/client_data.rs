pub struct ClientData {
    pub msn_login: String,
    pub msnp_version: i16,
}

impl ClientData {
    pub fn new(msn_login: String, msnp_version: i16) -> ClientData {
        return ClientData{ msn_login, msnp_version };
    }
}

