use std::{process::Command, str::FromStr};

use anyhow::anyhow;
use strum_macros::{Display, EnumString};

use crate::{msnp::{error::CommandError, notification::models::endpoint_guid::EndpointGuid, raw_command_parser::RawCommand}, shared::command::command::{get_split_part, parse_tr_id, SerializeMsnp}};

use super::ver::VerClient;

static OPERAND: &str = "USR";


pub struct UsrClient {
    pub tr_id: u128,
    pub auth_type: OperationTypeClient,
}


impl TryFrom<RawCommand> for UsrClient {
    type Error = CommandError;

    fn try_from(value: RawCommand) -> Result<Self, Self::Error> {
        UsrClient::from_str(value.get_command())
    }
}

impl FromStr for UsrClient {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split = s.split_whitespace().collect::<Vec<&str>>();
        let tr_id = parse_tr_id(&split)?;
        let auth_type = OperationTypeClient::from_command_slice(&split, s)?;

        Ok(Self {
            tr_id,
            auth_type,
        })
    }
}

#[derive(Display)]
pub enum OperationTypeClient {
    #[strum(serialize = "SSO")]
    Sso(SsoPhaseClient),
    Sha(),
}

impl OperationTypeClient {
    fn from_command_slice(command: &[&str], raw_cmd: &str) -> Result<Self, CommandError> {
    let raw_op_type= get_split_part(2, command, "", "sso_phase")?;
        match raw_op_type {
            "SSO" => Ok(OperationTypeClient::Sso(SsoPhaseClient::from_command_slice(command, raw_cmd)?)),
            "SHA" => Ok(OperationTypeClient::Sha()),
            _ => {
                Err(CommandError::ArgumentParseError { argument: raw_op_type.to_string(), command: raw_cmd.to_string(), source: anyhow!("Unknown operation type") })
            }
        }
    }
}

pub enum SsoPhaseClient {
    I {
        email_addr: String,
    },
    S {
        ticket_token: TicketToken,
        challenge: String,
        endpoint_guid: EndpointGuid,
    },
}

pub struct TicketToken(pub String);

impl std::fmt::Display for TicketToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t={}", self.0)
    }
}

impl FromStr for TicketToken {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let no_prefix = s.strip_prefix("t=").ok_or(Self::Err::ArgumentParseError { argument: s.to_string(), command: String::new(), source: anyhow!("Error stripping t= prefix from Ticket Token")})?;
        Ok(Self(no_prefix.to_string()))
    }
}

impl SsoPhaseClient {

    fn from_command_slice(split: &[&str], raw_cmd: &str) -> Result<Self, CommandError> {
        let raw_sso_phase = get_split_part(3, split, "", "sso_phase")?;

         match raw_sso_phase {
            "I" => {
                let email_addr = get_split_part(4, split, "", "email_addr")?.to_string();
                Ok(SsoPhaseClient::I { email_addr })
            },
            "S" => {
                let ticket_token = TicketToken::from_str(get_split_part(4, split, "", "ticket_token")?)?;
                    
                let challenge = get_split_part(5, split, "", "challenge")?.to_string(); 
                
                let endpoint_guid = EndpointGuid::from_str(get_split_part(6, split, raw_cmd, "endpoint_guid")?)?; 

                Ok(SsoPhaseClient::S {
                    ticket_token,
                    challenge,
                    endpoint_guid,
                })
            },
            _ => {
                Err(CommandError::ArgumentParseError { argument: raw_sso_phase.to_string(), command: raw_cmd.to_string(), source: anyhow!("Unknown sso phase") })
            }
        }
    }
}

pub enum OperationTypeServer {
    Sso(SsoPhaseServer),
    Ok {
        email_addr: String,
        verified: bool,
        unknown_arg: bool,
    },
}

impl core::fmt::Display for OperationTypeServer {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            OperationTypeServer::Sso(content) => {
                write!(f, "SSO {content}", content = content)
            },
            OperationTypeServer::Ok { email_addr, verified, unknown_arg } => {
                write!(f, "OK {email_addr} {verified} {unknown}", email_addr = email_addr, verified = verified.to_owned() as i32, unknown = unknown_arg.to_owned() as i32)
            }
        }
    }
}


pub enum SsoPhaseServer {
    S { policy: AuthPolicy, nonce: String },
}

impl core::fmt::Display for SsoPhaseServer{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            SsoPhaseServer::S{policy, nonce} => {
                write!(f, "{phase} {auth_policy} {nonce}", phase = "S", auth_policy = policy, nonce = nonce)
            }
        }
    }
}

#[derive(Display, EnumString)]
pub enum AuthPolicy {
    #[strum(serialize = "MBI_KEY_OLD")]
    MbiKeyOld,
}

pub struct UsrServer {
    tr_id: u128,
    auth_type: OperationTypeServer,
}

impl UsrServer {
    pub fn new(tr_id: u128, auth_type: OperationTypeServer) -> Self{
        Self {
            tr_id,
            auth_type
        }
    }
}

impl core::fmt::Display for UsrServer {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} {auth_type}\r\n", operand = OPERAND, tr_id = self.tr_id, auth_type = self.auth_type)

    }
}

impl SerializeMsnp for UsrServer {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}


#[cfg(test)]
mod tests {
    use crate::msnp::{error::CommandError, notification::command::usr::{OperationTypeClient, SsoPhaseClient}};

    use super::{AuthPolicy, OperationTypeServer, SsoPhaseServer, UsrClient, UsrServer};
    use std::str::FromStr;


    #[test]
    fn client_sso_i_des_success() {
        let usr1 = UsrClient::from_str("USR 3 SSO I login@test.com").unwrap();
        assert_eq!(3, usr1.tr_id);
        
        assert!(matches!(&usr1.auth_type, OperationTypeClient::Sso(_)));

        if let OperationTypeClient::Sso(content) = &usr1.auth_type {
            assert!(matches!(content, SsoPhaseClient::I { .. }));

            if let SsoPhaseClient::I { email_addr } = content {
                assert_eq!("login@test.com", email_addr);
            }

        } 

    }

    #[test]
    fn client_sso_i_des_failure() {
        let usr1 = UsrClient::from_str("USR 3 SSA I login@test.com");
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_2() {
        let usr1 = UsrClient::from_str("USR 3 SSO XY login@test.com");
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_3() {
        let usr1 = UsrClient::from_str("USR 4 SSO S t=ssotoken ???charabia");
        assert!(matches!(&usr1, Err(CommandError::MissingArgument { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_invalid_ticket_token() {
        let usr1 = UsrClient::from_str("USR 4 SSO S ssotoken ???charabia {55192CF5-588E-4ABE-9CDF-395B616ED85B}");
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_invalid_endpoint() {
        let usr1 = UsrClient::from_str("USR 4 SSO S t=ssotoken ???charabia 55192CF5-588E-4ABE-9CDF-395B616ED85B");
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_s_des_success() {
        let usr1 = UsrClient::from_str("USR 4 SSO S t=ssotoken ???charabia {55192CF5-588E-4ABE-9CDF-395B616ED85B}").unwrap();
        assert_eq!(4, usr1.tr_id);
        

        assert!(matches!(&usr1.auth_type, OperationTypeClient::Sso(_)));

        if let OperationTypeClient::Sso(content) = &usr1.auth_type {
            assert!(matches!(content, SsoPhaseClient::S { .. }));

            if let SsoPhaseClient::S { ticket_token, challenge, endpoint_guid } = content {
                assert_eq!("t=ssotoken", ticket_token.to_string());
                assert_eq!("???charabia", challenge);
                assert_eq!("{55192CF5-588E-4ABE-9CDF-395B616ED85B}", endpoint_guid.to_string());

            }

        } 

    }

    

    #[test]
    fn client_sha_des_success() {
        let usr1 = UsrClient::from_str("USR 4 SHA").unwrap();
        assert_eq!(4, usr1.tr_id);
        assert!(matches!(&usr1.auth_type, OperationTypeClient::Sha()));
    }

    #[test]
    fn server_sso_s_ser() {
        let usr = UsrServer { tr_id: 1, auth_type : OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "n0nce".to_string() }) };
        let ser = usr.to_string();
        assert_eq!("USR 1 SSO S MBI_KEY_OLD n0nce\r\n", ser);
    }

    #[test]
    fn server_ok_ser() {
        let usr = UsrServer { tr_id: 2, auth_type : OperationTypeServer::Ok { email_addr: "Xx-taytay-xX@hotmail.com".to_string(), verified: true, unknown_arg: false }};
        let ser = usr.to_string();
        assert_eq!("USR 2 OK Xx-taytay-xX@hotmail.com 1 0\r\n", ser);
    }



}