use strum_macros::{Display, EnumString};

struct UsrClient {
    tr_id: u128,
    auth_type: OperationTypeClient
}


#[derive(Display)]
pub enum OperationTypeClient {
    #[strum(serialize = "SSO")]
    Sso(SsoPhaseClient),
    Sha()
}

pub enum SsoPhaseClient {
    I{email_addr: String},
    S{ticket_token: String, challenge: String, endpoint_guid: String}
}

#[derive(Display)]
pub enum OperationTypeServer {
    #[strum(serialize = "SSO")]
    Sso(SsoPhaseServer),
    Ok{email_addr: String, verified: bool, unknown_arg: bool}
}

pub enum SsoPhaseServer {
    S{policy: AuthPolicy, nonce: String}
}

#[derive(Display, EnumString)]
pub enum AuthPolicy {

    #[strum(serialize = "MBI_KEY_OLD")]
    MbiKeyOld

}

struct UsrServer {
    tr_id: u128,
    auth_type: OperationTypeServer    

}