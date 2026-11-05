use std::{fmt::Display, str::FromStr};

use crate::shared::traits::{IntoBytes, TryFromRawCommand};
use crate::msnp::{error::CommandError, raw_command_parser::RawCommand};


pub struct CvrClient {
    pub tr_id: u128,
    pub region_code: u32,
    pub os_type: String,
    pub os_version: String,
    pub cpu_arch: String,
    pub msnp_lib_name: String,
    pub client_ver: String,
    pub client_name: String,
    pub email_addr: String,
}

impl CvrClient {
    pub fn new(
        tr_id: u128,
        region_code: u32,
        os_type: String,
        os_version: String,
        cpu_arch: String,
        msnp_lib_name: String,
        client_ver: String,
        client_name: String,
        email_addr: String,
    ) -> Self {
        Self {
            tr_id,
            region_code,
            os_type,
            os_version,
            cpu_arch,
            msnp_lib_name,
            client_ver,
            client_name,
            email_addr,
        }
    }
}


impl TryFromRawCommand for CvrClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {

        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_region_code = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "region_code".into(), 2))?;
        let region_code = u32::from_str_radix(raw_region_code.trim_start_matches("0x"), 16)?;

        let os_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "os_type".into(), 3))?;

        let os_version = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "os_version".into(), 4))?;

        let cpu_arch = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "cpu_arch".into(), 5))?;

        let msnp_lib_name = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "msnp_lib_name".into(), 6))?;

        let client_ver = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "client_ver".into(), 7))?;

        let client_name = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "client_name".into(), 8))?;

        let email_addr = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "email_addr".into(), 9))?;

        Ok(CvrClient {
            tr_id,
            region_code,
            os_type,
            os_version,
            cpu_arch,
            msnp_lib_name,
            client_ver,
            client_name,
            email_addr,
        })
    }
}

impl Display for CvrClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CVR {tr_id} {region_code} {os_type} {os_version} {cpu_arch} {msnp_lib_name} {client_ver} {client_name} {email_addr}\r\n",
            tr_id = self.tr_id,
            region_code = format!("0x{:04x}", self.region_code),
            os_type = self.os_type,
            os_version = self.os_version,
            cpu_arch = self.cpu_arch,
            msnp_lib_name = self.msnp_lib_name,
            client_ver = self.client_ver,
            client_name = self.client_name,
            email_addr = self.email_addr
        )
    }
}

impl IntoBytes for CvrClient {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

pub struct CvrServer {
    pub tr_id: u128,
    pub rec_client_ver: String,
    pub rec_client_ver2: String,
    pub min_client_ver: String,
    pub client_dl_url: String,
    pub client_info_url: String,
}

impl CvrServer {

    pub fn new(
        tr_id: u128,
        rec_client_ver: String,
        rec_client_ver2: String,
        min_client_ver: String,
        client_dl_url: String,
        client_info_url: String

    ) -> Self {
        Self {
            tr_id,
            rec_client_ver,
            rec_client_ver2,
            min_client_ver,
            client_dl_url,
            client_info_url,
        }
    }
}

impl Display for CvrServer {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} {rec_client_ver} {rec_client_ver2} {min_client_ver} {client_dl_url} {client_info_url}\r\n", 
            operand =  "CVR",
            tr_id = self.tr_id,
            rec_client_ver = self.rec_client_ver,
            rec_client_ver2 = self.rec_client_ver2,
            min_client_ver = self.min_client_ver,
            client_dl_url = self.client_dl_url,
            client_info_url = self.client_info_url
        )
    }
}

impl TryFromRawCommand for CvrServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let rec_client_ver = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "rec_client_ver".into(), 2))?;
        let rec_client_ver2 = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "rec_client_ver2".into(), 3))?;
        let min_client_ver = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "min_client_ver".into(), 4))?;
        let client_dl_url = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "client_dl_url".into(), 5))?;
        let client_info_url = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "client_info_url".into(), 6))?;

        Ok(Self {
            tr_id,
            rec_client_ver,
            rec_client_ver2,
            min_client_ver,
            client_dl_url,
            client_info_url,
        })
    }
}

impl IntoBytes for CvrServer {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use crate::msnp::notification::command::cvr::{CvrClient, CvrServer};
    use crate::msnp::raw_command_parser::{RawCommand, ServerCommandParser};
    use crate::shared::traits::{IntoBytes, TryFromRawCommand};
    use std::str::FromStr;


    #[test]
    fn deser_test() {
       let cvr =  CvrClient::try_from_raw(RawCommand::from_str("CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeontest3@shlasouf.local").unwrap()).unwrap();
    }

    #[test]
    fn client_serializes_the_region_code_with_the_0x_prefix() {
        let cvr = CvrClient::new(
            2,
            0x0409,
            "winnt".to_string(),
            "6.2.0".to_string(),
            "i386".to_string(),
            "MSNMSGR".to_string(),
            "14.0.8117.0416".to_string(),
            "msmsgs".to_string(),
            "aeontest3@shlasouf.local".to_string(),
        );

        let bytes = cvr.into_bytes();

        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            "CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeontest3@shlasouf.local\r\n"
        );
    }

    #[test]
    fn client_round_trips_through_the_parser() {
        let cvr = CvrClient::new(
            2,
            0x0409,
            "winnt".to_string(),
            "6.2.0".to_string(),
            "i386".to_string(),
            "MSNMSGR".to_string(),
            "14.0.8117.0416".to_string(),
            "msmsgs".to_string(),
            "aeontest3@shlasouf.local".to_string(),
        );

        let mut parser = crate::msnp::raw_command_parser::RawCommandParser::new();
        let raw = parser.parse_message(&cvr.into_bytes()).unwrap().pop().unwrap();
        let parsed = CvrClient::try_from_raw(raw).unwrap();

        assert_eq!(parsed.tr_id, 2);
        assert_eq!(parsed.region_code, 0x0409);
        assert_eq!(parsed.email_addr, "aeontest3@shlasouf.local");
        assert_eq!(parsed.client_ver, "14.0.8117.0416");
    }

    #[test]
    fn server_deserializes_the_recommendation_reply() {
        let mut parser = ServerCommandParser::new();
        let raw = parser
            .parse_message(b"CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n")
            .unwrap()
            .pop()
            .unwrap();

        let parsed = CvrServer::try_from_raw(raw).unwrap();

        assert_eq!(parsed.tr_id, 2);
        assert_eq!(parsed.rec_client_ver, "14.0.8117.0416");
        assert_eq!(parsed.rec_client_ver2, "14.0.8117.0416");
        assert_eq!(parsed.min_client_ver, "14.0.8117.0416");
        assert_eq!(parsed.client_dl_url, "localhost");
        assert_eq!(parsed.client_info_url, "localhost");
    }

    #[test]
    fn server_serializes_the_recommendation_reply_byte_exact() {
        let cvr = CvrServer::new(
            2,
            "14.0.8117.0416".to_string(),
            "14.0.8117.0416".to_string(),
            "14.0.8117.0416".to_string(),
            "localhost".to_string(),
            "localhost".to_string(),
        );

        let bytes = cvr.into_bytes();

        assert_eq!(
            String::from_utf8(bytes).unwrap(),
            "CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n"
        );
    }
}