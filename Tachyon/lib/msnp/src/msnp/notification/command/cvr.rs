use std::{fmt::Display, str::FromStr};

use crate::{msnp::{error::CommandError, raw_command_parser::RawCommand}, shared::command::command::{parse_tr_id, split_raw_command, SerializeMsnp}};


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


impl TryFrom<RawCommand> for CvrClient {
    type Error = CommandError;

    fn try_from(value: RawCommand) -> Result<Self, Self::Error> {
        CvrClient::from_str(value.get_command())
    }
}

impl FromStr for CvrClient {
    type Err = CommandError;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let split = split_raw_command(command, 10)?;
        let tr_id = parse_tr_id(&split)?;
        
        let region_code_as_str = split.get(2).expect("region code to be present").trim_start_matches("0x");
        let region_code = u32::from_str_radix(region_code_as_str, 16).map_err(|e| Self::Err::ArgumentParseError {argument: region_code_as_str.to_string() , command: command.to_string(), source: e.into()})?;

        let os_type = split.get(3).expect("os type to be present").to_string();

        let os_version = split.get(4).expect("os version to be present").to_string();

        let cpu_arch: String = split.get(5).expect("cpu arch to be present").to_string();

        let msnp_lib_name: String = split.get(6).expect("msnp_lib_name to be present").to_string();

        let client_ver: String = split.get(7).expect("client_ver to be present").to_string();

        let client_name: String = split.get(8).expect("client_name to be present").to_string();

        let email_addr: String = split.get(9).expect("email_address to be present").to_string();

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

impl FromStr for CvrServer {
    type Err = CommandError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!()
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

impl SerializeMsnp for CvrServer {

    fn serialize_msnp(&self) -> Vec<u8> {
        self.to_string().as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::msnp::notification::command::cvr::CvrClient;


    #[test]
    fn deser_test() {
       let cvr =  CvrClient::from_str("CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeontest3@shlasouf.local").unwrap();
    }

}