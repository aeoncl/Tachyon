#[derive(Clone, Debug)]

pub struct SwitchboardInitEventContent {
    pub ip_address: String,
    pub port: u16,
    pub invite_passport: String,
    pub invite_name: String,
    pub session_id: String,
    pub ticket: String,
}