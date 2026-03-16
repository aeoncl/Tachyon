use crate::msnp::error::CommandError;
use crate::msnp::notification::models::ip_address::IpAddress;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::models::auth_method::AuthenticationMethod;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Display, EnumString)]
pub enum ClientRequestType {
    #[strum(serialize = "SB")]
    Switchboard
}

pub struct XfrClient {
    pub tr_id: u128,
    pub request_type: ClientRequestType
}

impl XfrClient {

    pub fn new(tr_id: u128, request_type: ClientRequestType) -> Self {
        Self {
            tr_id,
            request_type
        }
    }

    pub fn get_response_for(&self, address: IpAddress, auth_token: String) -> XfrServer {
        XfrServer::new_switchboard(self.tr_id, address, auth_token)
    }
}

impl TryFromRawCommand for XfrClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_request_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "request_type".into(), 2))?;
        let request_type = ClientRequestType::from_str(&raw_request_type)?;

        Ok(Self::new(tr_id, request_type))
    }

}

impl IntoBytes for XfrClient {

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl Display for XfrClient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "XFR {} {}\r\n", self.tr_id, self.request_type)
    }
}

#[derive(Display)]
pub enum ServerRequestType {
    Switchboard {
        address: IpAddress,
        authentication_method: AuthenticationMethod,
        auth_token: String
    },

    NotificationServer {
        address: IpAddress,
        //Always 0
        unknown: u32,
        current_address: IpAddress
    }
}

pub struct XfrServer {
    pub tr_id: u128,
    pub request_type: ServerRequestType
}

impl XfrServer {
    fn new_switchboard(tr_id: u128, address: IpAddress, auth_token: String) -> Self {
        Self {
            tr_id,
            request_type: ServerRequestType::Switchboard {
                address,
                authentication_method: AuthenticationMethod::CKI,
                auth_token
            }
        }
    }

    fn new_notification_server(tr_id: u128, address: IpAddress, current_address: IpAddress) -> Self {
        Self {
            tr_id,
            request_type: ServerRequestType::NotificationServer {
                address,
                unknown: 0,
                current_address
            }
        }
    }
}

impl TryFromRawCommand for XfrServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        todo!()
    }

}

impl IntoBytes for XfrServer {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl Display for XfrServer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.request_type {
            ServerRequestType::Switchboard { address, authentication_method, auth_token } => {
                write!(f,"XFR {tr_id} SB {address} {authentication_method} {auth_token}\r\n", tr_id = self.tr_id, address = address, authentication_method = authentication_method, auth_token = auth_token)
            }
            ServerRequestType::NotificationServer { address, unknown, current_address } => {
                write!(f,"XFR {tr_id} NS {address} {unknown} {current_address}\r\n", tr_id = self.tr_id, address = address, unknown = unknown, current_address = current_address)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::msnp::notification::command::xfr::{ClientRequestType, XfrClient, XfrServer};
    use crate::msnp::raw_command_parser::RawCommand;
    use crate::shared::traits::TryFromRawCommand;
    use std::str::FromStr;

    #[test]
    fn request_deserialization_client() {
        let xfr = XfrClient::try_from_raw(RawCommand::from_str("XFR 12 SB\r\n").unwrap()).unwrap();

        assert_eq!(12, xfr.tr_id);
        assert!(matches!(xfr.request_type, ClientRequestType::Switchboard));

    }

    #[test]
    fn request_serialization_client() {
        let xfr = XfrClient::new(12, ClientRequestType::Switchboard);
        let ser = xfr.to_string();

        assert_eq!("XFR 12 SB\r\n", ser);
    }

    #[test]
    fn request_serialization_server_ns() {
        let xfr_server = XfrServer::new_notification_server(12, "127.0.0.1:1862".parse().unwrap(), "127.0.0.1:1863".parse().unwrap());
        let ser = xfr_server.to_string();
        assert_eq!("XFR 12 NS 127.0.0.1:1862 0 127.0.0.1:1863\r\n", ser);
    }

    #[test]
    fn request_serialization_server_sb() {
        let xfr_server = XfrServer::new_switchboard(12, "127.0.0.1:1864".parse().unwrap(), "4uth_t0k3n".to_string());
        let ser = xfr_server.to_string();
        assert_eq!("XFR 12 SB 127.0.0.1:1864 CKI 4uth_t0k3n\r\n", ser);
    }
}