use std::{fmt::Display, str::FromStr};

use crate::{msnp::{error::CommandError, raw_command_parser::RawCommand}};
use crate::shared::traits::MSNPCommand;


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


impl MSNPCommand for CvrClient {
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

    fn into_bytes(self) -> Vec<u8> {
        todo!()
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

impl MSNPCommand for CvrServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::msnp::notification::command::cvr::CvrClient;
    use crate::msnp::raw_command_parser::RawCommand;
    use crate::shared::traits::MSNPCommand;


    #[test]
    fn deser_test() {
       let cvr =  CvrClient::try_from_raw(RawCommand::from_str("CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeontest3@shlasouf.local").unwrap()).unwrap();
    }

}