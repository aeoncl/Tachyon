use std::{fmt::Display, str::FromStr};

use byteorder::{ByteOrder, LittleEndian};
use thiserror::Error;


#[derive(Clone, Debug)]
pub struct Puid {

    bytes: [u8; 8]

}

impl Puid {
    pub fn new(bytes: [u8; 8]) -> Puid {
        Puid { bytes }
    }

    pub fn get_least_significant_bytes(&self) -> u32 {
        let lsb = &self.bytes[4..8];
        LittleEndian::read_u32(&lsb)
    }

    pub fn get_most_significant_bytes(&self) -> u32 {
        let msb = &self.bytes[0..4];
        LittleEndian::read_u32(&msb)
    }

    pub fn to_decimal(&self) -> i64 {
        LittleEndian::read_u64(&self.bytes[..]) as i64
    }
}

impl From<&Uuid> for Puid {

    fn from(value: &Uuid) -> Self {
        Puid::new(value.get_least_significant_bytes_as_array())
    }
}

impl Display for Puid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = format!("{}", LittleEndian::read_u64(&self.bytes));
        write!(f, "{}", out)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] uuid::Error)
}


#[derive(Clone, Debug, Default, Hash, Eq)]

pub struct Uuid {
    uuid : uuid::Uuid
}

impl Uuid {

    pub fn new() -> Uuid{
        return Uuid{ uuid: uuid::Uuid::new_v4() };
    }

    pub fn from_seed(seed: &str) -> Uuid {
        return Uuid { uuid: uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, seed.as_bytes()) }
    }

    pub fn nil() -> Uuid {
        return Uuid { uuid: uuid::Uuid::nil() };
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

    pub fn get_puid(&self) -> Puid {
        return self.into();
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


impl From<uuid::Uuid> for Uuid {
    fn from(value: uuid::Uuid) -> Self {
        Uuid { uuid: value }
    }
}

impl FromStr for Uuid {

    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let uuid = uuid::Uuid::from_str(s)?;
        return Result::Ok(Uuid { uuid });
    }
}

impl Display for Uuid {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let out = self.uuid.to_string().to_uppercase();
            return write!(f, "{}", out);
    }
}

impl PartialEq for Uuid {
    fn eq(&self, other: &Self) -> bool {
       self.uuid == other.uuid
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::Uuid;

    #[test]
    fn test() {
        let uuid = Uuid::new();
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
        //TODO add assertions
    }

    #[test]
    fn parse_machine_guid() {

        let machine_guid = String::from("F52973B6-C926-4BAD-9BA8-7C1E840E4AB0");
        let uuid = Uuid::from_str(machine_guid.as_str()).unwrap();
        let uuid_serialied = uuid.to_string();

        assert_eq!(machine_guid, uuid_serialied);
    }
}
