use std::collections::VecDeque;
use std::str::FromStr;

use anyhow::anyhow;
use strum_macros::{Display, EnumString};

use crate::msnp::{error::CommandError, notification::models::endpoint_guid::EndpointGuid, raw_command_parser::RawCommand};
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::ticket_token::TicketToken;
use crate::shared::traits::{IntoBytes, TryFromRawCommand, TryFromSplit};

static OPERAND: &str = "USR";


pub struct UsrClient {
    pub tr_id: u128,
    pub auth_type: AuthOperationTypeClient,
}

impl IntoBytes for UsrClient {
    fn into_bytes(self) -> Vec<u8> {
        match self.auth_type {
            AuthOperationTypeClient::Sso(phase) => format!("USR {} SSO {}\r\n", self.tr_id, phase).into_bytes(),
            AuthOperationTypeClient::Sha(phase) => format!("USR {} SHA {}\r\n", self.tr_id, phase).into_bytes(),
        }
    }
}


impl TryFromRawCommand for UsrClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let auth_type = AuthOperationTypeClient::try_from_split(split, &raw.command)?;

        Ok(Self {
            tr_id,
            auth_type,
        })

    }
}

#[derive(Display)]
pub enum AuthOperationTypeClient {
    #[strum(serialize = "SSO")]
    Sso(SsoPhaseClient),
    #[strum(serialize = "SHA")]
    Sha(ShaPhaseClient),
}

impl TryFromSplit for AuthOperationTypeClient {
    type Err = CommandError;

    fn try_from_split(mut split: VecDeque<String>, command: &str) -> Result<Self, Self::Err> {
        let raw_op_type = split.pop_front().ok_or(CommandError::MissingArgument(command.to_string(), "operation_type".into(), 2))?;

        match raw_op_type.as_str() {
            "SSO" => Ok(AuthOperationTypeClient::Sso(SsoPhaseClient::try_from_split(split, command)?)),
            "SHA" => Ok(AuthOperationTypeClient::Sha(ShaPhaseClient::try_from_split(split, command)?)),
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

impl core::fmt::Display for ShaPhaseClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShaPhaseClient::A { circle_ticket } => write!(f, "A {circle_ticket}"),
        }
    }
}

impl TryFromSplit for ShaPhaseClient{
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

impl core::fmt::Display for SsoPhaseClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SsoPhaseClient::I { email_addr } => write!(f, "I {email_addr}"),
            SsoPhaseClient::S { ticket_token, challenge, endpoint_guid } => {
                write!(f, "S {ticket_token} {challenge} {endpoint_guid}")
            }
        }
    }
}


impl TryFromSplit for SsoPhaseClient {
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
    pub tr_id: u128,
    pub auth_type: OperationTypeServer,
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

impl TryFromRawCommand for UsrServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_operation_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "operation_type".into(), 2))?;
        let auth_type = match raw_operation_type.as_str() {
            "SSO" => {
                let raw_phase = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "sso_phase".into(), 3))?;
                match raw_phase.as_str() {
                    "S" => {
                        let raw_policy = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "auth_policy".into(), 4))?;
                        let policy = AuthPolicy::from_str(&raw_policy).map_err(|e| CommandError::ArgumentParseError {
                            argument: raw_policy.clone(),
                            command: raw.command.clone(),
                            source: anyhow!("Unknown auth policy: {e}"),
                        })?;
                        let nonce = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "nonce".into(), 5))?;
                        OperationTypeServer::Sso(SsoPhaseServer::S { policy, nonce })
                    }
                    _ => {
                        return Err(CommandError::ArgumentParseError {
                            argument: raw_phase,
                            command: raw.command,
                            source: anyhow!("Unknown sso phase"),
                        })
                    }
                }
            }
            "OK" => {
                let raw_email = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "email_addr".into(), 3))?;
                let email_addr = EmailAddress::from_str(&raw_email)?;

                let raw_verified = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "verified".into(), 4))?;
                let verified = i32::from_str(&raw_verified)?;

                let raw_unknown_arg = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "unknown_arg".into(), 5))?;
                let unknown_arg = i32::from_str(&raw_unknown_arg)?;

                OperationTypeServer::Ok {
                    email_addr,
                    verified: verified != 0,
                    unknown_arg: unknown_arg != 0,
                }
            }
            _ => {
                return Err(CommandError::ArgumentParseError {
                    argument: raw_operation_type,
                    command: raw.command,
                    source: anyhow!("Unknown operation type"),
                })
            }
        };

        Ok(Self { tr_id, auth_type })
    }

}

impl IntoBytes for UsrServer {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::notification::models::endpoint_guid::EndpointGuid;
    use crate::msnp::raw_command_parser::RawCommand;
    use crate::msnp::{error::CommandError, notification::command::usr::{AuthOperationTypeClient, SsoPhaseClient}};
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::ticket_token::TicketToken;
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};

    use super::{AuthPolicy, OperationTypeServer, SsoPhaseServer, UsrClient, UsrServer};

    #[test]
    fn client_sso_i_des_success() {
        let usr1 = UsrClient::try_from_raw(RawCommand::from_str("USR 3 SSO I login@test.com").unwrap()).unwrap();
        assert_eq!(3, usr1.tr_id);

        assert!(matches!(&usr1.auth_type, AuthOperationTypeClient::Sso(_)));

        if let AuthOperationTypeClient::Sso(content) = &usr1.auth_type {
            assert!(matches!(content, SsoPhaseClient::I { .. }));

            if let SsoPhaseClient::I { email_addr } = content {
                assert_eq!("login@test.com", email_addr.as_str());
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


        assert!(matches!(&usr1.auth_type, AuthOperationTypeClient::Sso(_)));

        if let AuthOperationTypeClient::Sso(content) = &usr1.auth_type {
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
        assert!(matches!(&usr1.auth_type, AuthOperationTypeClient::Sha(_)));
    }

    #[test]
    fn server_sso_s_ser() {
        let usr = UsrServer { tr_id: 1, auth_type : OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "n0nce".to_string() }) };
        let ser = usr.to_string();
        assert_eq!("USR 1 SSO S MBI_KEY_OLD n0nce\r\n", ser);
    }

    #[test]
    fn server_ok_ser() {
        let usr = UsrServer { tr_id: 2, auth_type : OperationTypeServer::Ok { email_addr: EmailAddress::from_str("Xx-taytay-xX@hotmail.com").unwrap(), verified: true, unknown_arg: false }};
        let ser = usr.to_string();
        assert_eq!("USR 2 OK Xx-taytay-xX@hotmail.com 1 0\r\n", ser);
    }

    #[test]
    fn server_sso_challenge_des() {
        let mut parser = crate::msnp::raw_command_parser::ServerCommandParser::new();
        let raw = parser
            .parse_message(b"USR 3 SSO S MBI_KEY_OLD LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU\r\n")
            .unwrap()
            .pop()
            .unwrap();

        let usr = UsrServer::try_from_raw(raw).unwrap();

        assert_eq!(usr.tr_id, 3);
        let OperationTypeServer::Sso(SsoPhaseServer::S { policy, nonce }) = usr.auth_type else {
            panic!("expected the SSO challenge, got {usr}");
        };
        assert!(matches!(policy, AuthPolicy::MbiKeyOld));
        assert_eq!(nonce, "LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU");
    }

    #[test]
    fn server_ok_des() {
        let mut parser = crate::msnp::raw_command_parser::ServerCommandParser::new();
        let raw = parser
            .parse_message(b"USR 4 OK Xx-taytay-xX@hotmail.com 1 0\r\n")
            .unwrap()
            .pop()
            .unwrap();

        let usr = UsrServer::try_from_raw(raw).unwrap();

        assert_eq!(usr.tr_id, 4);
        let OperationTypeServer::Ok { email_addr, verified, unknown_arg } = usr.auth_type else {
            panic!("expected the OK reply, got {usr}");
        };
        assert_eq!(email_addr.as_str(), "Xx-taytay-xX@hotmail.com");
        assert!(verified);
        assert!(!unknown_arg);
    }

    #[test]
    fn client_sso_i_ser() {
        let usr = UsrClient {
            tr_id: 3,
            auth_type: AuthOperationTypeClient::Sso(SsoPhaseClient::I {
                email_addr: EmailAddress::from_str("login@test.com").unwrap(),
            }),
        };

        assert_eq!(usr.into_bytes(), b"USR 3 SSO I login@test.com\r\n");
    }

    #[test]
    fn client_sso_s_ser_round_trips() {
        let usr = UsrClient {
            tr_id: 4,
            auth_type: AuthOperationTypeClient::Sso(SsoPhaseClient::S {
                ticket_token: TicketToken::from_str("t=ssotoken").unwrap(),
                challenge: "challenge".to_string(),
                endpoint_guid: EndpointGuid::from_str("{C8D73CAE-A6F5-491B-A74B-A85E657CFC2C}").unwrap(),
            }),
        };

        let bytes = usr.into_bytes();
        assert_eq!(
            String::from_utf8(bytes.clone()).unwrap(),
            "USR 4 SSO S t=ssotoken challenge {C8D73CAE-A6F5-491B-A74B-A85E657CFC2C}\r\n"
        );

        let mut parser = crate::msnp::raw_command_parser::RawCommandParser::new();
        let raw = parser.parse_message(&bytes).unwrap().pop().unwrap();
        let parsed = UsrClient::try_from_raw(raw).unwrap();
        let AuthOperationTypeClient::Sso(SsoPhaseClient::S { ticket_token, .. }) = parsed.auth_type else {
            panic!("expected the S phase");
        };
        assert_eq!(ticket_token.as_str(), "ssotoken");
    }

}
