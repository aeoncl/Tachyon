use std::collections::VecDeque;
use std::str::FromStr;

use anyhow::anyhow;
use strum_macros::{Display, EnumString};

use crate::msnp::{error::CommandError, notification::models::endpoint_guid::EndpointGuid, raw_command_parser::RawCommand};
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::ticket_token::TicketToken;
use crate::shared::traits::{MSNPCommand, MSNPCommandPart};


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::{error::CommandError, notification::command::usr::{OperationTypeClient, SsoPhaseClient}};
    use crate::msnp::raw_command_parser::RawCommand;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::traits::MSNPCommand;

    use super::{AuthPolicy, OperationTypeServer, SsoPhaseServer, UsrClient, UsrServer};

    #[test]
    fn client_sso_i_des_success() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 3 SSO I login@test.com").unwrap()).unwrap();
        assert_eq!(3, usr1.tr_id);

        assert!(matches!(&usr1.auth_type, OperationTypeClient::Sso(_)));

        if let OperationTypeClient::Sso(content) = &usr1.auth_type {
            assert!(matches!(content, SsoPhaseClient::I { .. }));

            if let SsoPhaseClient::I { email_addr } = content {
                assert_eq!("login@test.com", &email_addr.0);
            }

        }

    }

    #[test]
    fn client_sso_i_des_failure() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 3 SSA I login@test.com").unwrap());
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_2() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 3 SSO XY login@test.com").unwrap());
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_3() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 4 SSO S t=ssotoken ???charabia").unwrap());
        assert!(matches!(&usr1, Err(CommandError::MissingArgument { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_invalid_ticket_token() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 4 SSO S ssotoken ???charabia {55192CF5-588E-4ABE-9CDF-395B616ED85B}").unwrap());
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_i_des_failure_invalid_endpoint() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 4 SSO S t=ssotoken ???charabia 55192CF5-588E-4ABE-9CDF-395B616ED85B").unwrap());
        assert!(matches!(&usr1, Err(CommandError::ArgumentParseError { .. })));
    }

    #[test]
    fn client_sso_s_des_success() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 4 SSO S t=ssotoken ???charabia {55192CF5-588E-4ABE-9CDF-395B616ED85B}").unwrap()).unwrap();
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
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 4 SHA A circleticket").unwrap()).unwrap();
        assert_eq!(4, usr1.tr_id);
        assert!(matches!(&usr1.auth_type, OperationTypeClient::Sha(_)));
    }

    #[test]
    fn server_sso_s_ser() {
        let usr = UsrServer { tr_id: 1, auth_type : OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "n0nce".to_string() }) };
        let ser = usr.to_string();
        assert_eq!("USR 1 SSO S MBI_KEY_OLD n0nce\r\n", ser);
    }

    #[test]
    fn server_ok_ser() {
        let usr = UsrServer { tr_id: 2, auth_type : OperationTypeServer::Ok { email_addr: EmailAddress("Xx-taytay-xX@hotmail.com".to_string()), verified: true, unknown_arg: false }};
        let ser = usr.to_string();
        assert_eq!("USR 2 OK Xx-taytay-xX@hotmail.com 1 0\r\n", ser);
    }



}

static OPERAND: &str = "USR";


pub struct UsrClient {
    pub tr_id: u128,
    pub auth_type: OperationTypeClient,
}


impl MSNPCommand for UsrClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let auth_type = OperationTypeClient::try_from_split(split, &raw.command)?;

        Ok(Self {
            tr_id,
            auth_type,
        })

    }

    fn into_bytes(self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Display)]
pub enum OperationTypeClient {
    #[strum(serialize = "SSO")]
    Sso(SsoPhaseClient),
    #[strum(serialize = "SHA")]
    Sha(ShaPhaseClient),
}

impl MSNPCommandPart for OperationTypeClient {
    type Err = CommandError;

    fn try_from_split(mut split: VecDeque<String>, command: &str) -> Result<Self, Self::Err> {
        let raw_op_type = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "operation_type".into(), 2))?;

        match raw_op_type.as_str() {
            "SSO" => Ok(OperationTypeClient::Sso(SsoPhaseClient::try_from_split(split, command)?)),
            "SHA" => Ok(OperationTypeClient::Sha(ShaPhaseClient::try_from_split(split, command)?)),
            _ => {
                Err(CommandError::ArgumentParseError { argument: raw_op_type.to_string(), command: command.to_string(), source: anyhow!("Unknown operation type") })
            }
        }
    }
}

pub enum ShaPhaseClient {
    A {
      circle_ticket: String
    }
}

impl MSNPCommandPart for ShaPhaseClient{
    type Err = CommandError;

    fn try_from_split(mut split: VecDeque<String>, command: &str) -> Result<Self, Self::Err> where Self: Sized {
        let raw_sha_phase = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "sha_phase".into(), 3))?;
        match raw_sha_phase.as_str() {
            "A" => {
                let circle_ticket = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "sha_phase".into(), 4))?;
                Ok(ShaPhaseClient::A {
                    circle_ticket
                })
            },
            _ => {
                Err(CommandError::ArgumentParseError { argument: raw_sha_phase.to_string(), command: command.to_string(), source: anyhow!("Unknown sha phase") })
            }
        }

    }
}

pub enum SsoPhaseClient {
    I {
        email_addr: EmailAddress,
    },
    S {
        ticket_token: TicketToken,
        challenge: String,
        endpoint_guid: EndpointGuid,
    },
}


impl MSNPCommandPart for SsoPhaseClient {
    type Err = CommandError;

    fn try_from_split(mut split: VecDeque<String>, command: &str) -> Result<Self, Self::Err> {
        let raw_sso_phase = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "sso_phase".into(), 3))?;

        match raw_sso_phase.as_str() {
            "I" => {
                let email_addr = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "email_addr".into(), 4))?;
                Ok(SsoPhaseClient::I { email_addr: EmailAddress::from_str(&email_addr)? })
            },
            "S" => {
                let raw_token = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "ticket_token".into(), 5))?;
                let ticket_token = TicketToken::from_str(&raw_token)?;

                let challenge = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "challenge".into(), 6))?;

                let raw_endpoint_guid = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "endpoint_guid".into(), 7))?;
                let endpoint_guid = EndpointGuid::from_str(&raw_endpoint_guid)?;

                Ok(SsoPhaseClient::S {
                    ticket_token,
                    challenge,
                    endpoint_guid,
                })
            },
            _ => {
                Err(CommandError::ArgumentParseError { argument: raw_sso_phase.to_string(), command: command.to_string(), source: anyhow!("Unknown sso phase") })
            }
        }

    }
}

pub enum OperationTypeServer {
    Sso(SsoPhaseServer),
    Ok {
        email_addr: EmailAddress,
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

impl MSNPCommand for UsrServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
