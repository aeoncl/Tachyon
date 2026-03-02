use std::fmt::{write, Display, Formatter};
use std::net::Ipv4Addr;
use std::str::FromStr;
use anyhow::anyhow;
use crate::msnp::error::CommandError;

pub struct IpAddress {
    pub ip: Ipv4Addr,
    pub port: u32,
}

impl IpAddress {
    pub fn new(ip: Ipv4Addr, port: u32) -> Self {
        Self { ip, port }
    }
}

impl FromStr for IpAddress {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(anyhow!("Ip Address should be in format '192.168.1.1:0000', got: {}", s));
        }
        
        let ip = Ipv4Addr::from_str(parts[0])?;
        let port = u32::from_str(parts[1])?;
        Ok(Self { ip, port })
    }
}

impl Display for IpAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.ip, self.port)
    }
}