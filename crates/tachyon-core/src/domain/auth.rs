#[derive(Hash, Eq, PartialEq)]
pub struct TachyonToken(String);

pub enum RestoreOutcome {
    Success,
    SoftLogout,
    Logout,
}

pub struct InteractiveAuthStarted {
    pub auth_url: String,
    pub csrf_token: String
}
