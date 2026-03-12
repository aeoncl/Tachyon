use strum_macros::{Display, EnumString};

#[derive(Display, EnumString)]
pub enum AuthenticationMethod {
    #[strum(serialize = "CKI")]
    CKI
}

impl Default for AuthenticationMethod {
    fn default() -> Self {
        AuthenticationMethod::CKI
    }
}