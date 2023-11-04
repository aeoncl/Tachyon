use std::{fmt::Display, str::FromStr};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use uuid::Uuid;

use crate::models::notification::error::MSNPErrorCode;
use crate::models::tachyon_error::TachyonError;

#[derive(Clone, Debug)]
pub struct PUID {

    bytes: [u8; 8]

}

impl PUID {
    pub fn new(bytes: [u8; 8]) -> PUID {
        return PUID { bytes };
    }

    pub fn get_least_significant_bytes(&self) -> u32 {
        let lsb = &self.bytes[4..8];
        return LittleEndian::read_u32(&lsb);
    }

    pub fn get_most_significant_bytes(&self) -> u32 {
        let msb = &self.bytes[0..4];
        return LittleEndian::read_u32(&msb);
    }
}


impl Display for PUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = format!("{}", LittleEndian::read_u64(&self.bytes));
        return write!(f, "{}", out);   
    }
}

#[derive(Clone, Debug)]

pub struct UUID {

    uuid : Uuid

}

impl UUID {

    pub fn new() -> UUID{
        return UUID{ uuid: Uuid::new_v4() };
    }

    pub fn from_string(s: &str) -> UUID {
        return UUID { uuid: Uuid::new_v5(&Uuid::NAMESPACE_OID, s.as_bytes()) }
    }

    pub fn parse(s: &str) -> Result<UUID, TachyonError> {
        return Ok(Self::from_uuid(Uuid::parse_str(s)?));
    }

    pub fn from_uuid(uuid: Uuid) -> UUID{
        return UUID { uuid };
    }

    pub fn nil() -> UUID {
        return UUID { uuid: Uuid::nil() };
    }

    fn get_least_significant_bytes_as_array(&self) -> [u8; 8] {
        let bytes = self.uuid.as_bytes();
        let lsb = &bytes[8..16];
        return lsb.try_into().unwrap();
    }
    
    pub fn get_least_significant_bytes(&self) -> u64 {
        let bytes = self.uuid.as_bytes();
        let lsb = &bytes[8..16];
        return LittleEndian::read_u64(&lsb);
    }

    pub fn get_least_significant_bytes_as_hex(&self) -> String {
        return format!("{:x}", self.get_least_significant_bytes());
    }

    fn get_most_significant_bytes_as_array(&self) -> [u8; 8] {
        let bytes = self.uuid.as_bytes();
        let msb = &bytes[0..8];
        return msb.try_into().unwrap();
    }

    pub fn get_most_significant_bytes(&self) -> u64 {
        let bytes = self.uuid.as_bytes();
        let msb = &bytes[0..8];
        return LittleEndian::read_u64(&msb);
    }

    pub fn get_most_significant_bytes_as_hex(&self) -> String {
        return format!("{:x}", self.get_most_significant_bytes());
    }

    pub fn get_puid(&self) -> PUID {
        return PUID::new(self.get_least_significant_bytes_as_array());
    }

    pub fn to_hex_string(&self) -> String {
        return format!("{:x}", self.uuid.as_u128());
    }

    pub fn to_decimal_cid_string(&self) -> String {
        return format!("{}", self.get_most_significant_bytes());
    }

    pub fn to_decimal_cid(&self) -> i64 {
        return self.get_most_significant_bytes() as i64;
    }

    pub fn to_hex_cid(&self) -> String {
        return format!("{:x}", self.get_most_significant_bytes());
    }
}

impl FromStr for UUID {

    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = Uuid::from_str(s).unwrap();
        return Result::Ok(UUID { uuid: uuid});
    }
}

impl Display for UUID {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

            let out = self.uuid.to_string().to_uppercase();

            return write!(f, "{}", out);
    }
}


#[cfg(test)]
mod tests {
    use super::UUID;

    #[test]
    fn test() {
        let uuid = UUID::new();
        let test = uuid.to_string();
        let test1 = uuid.get_least_significant_bytes();
        let test2 = uuid.get_most_significant_bytes();
        let test10 = uuid.to_hex_string();
        let test3 = uuid.to_decimal_cid();
        let test4 = uuid.to_hex_cid();
        let test5 = uuid.get_puid();
        let test6 = test5.get_least_significant_bytes();
        let test7 = test5.get_most_significant_bytes();

        let testfinal = 0;
    }

    #[test]
    fn parse_machine_guid() {

        let machine_guid = String::from("F52973B6-C926-4BAD-9BA8-7C1E840E4AB0");
        let uuid = UUID::parse(machine_guid.as_str()).unwrap();
        let uuid_serialied = uuid.to_string();

        assert_eq!(machine_guid, uuid_serialied);
    }
}
