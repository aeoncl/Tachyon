pub struct TachyonToken(String);

pub enum RestoreOutcome {
    Success,
    SoftLogout,
    Logout,
}
