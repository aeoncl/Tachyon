use std::fmt::Display;
use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::notification::models::endpoint_guid::EndpointGuid;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::uuid::Uuid;

#[derive(Clone, Debug)]
pub struct EndpointId {
    pub email_addr: EmailAddress,
    pub endpoint_guid: Option<EndpointGuid>
}

impl EndpointId {
    pub fn new(email_addr: EmailAddress, endpoint_guid: Option<EndpointGuid>) -> Self {
        Self{
            email_addr,
            endpoint_guid,
        }
    }

    pub fn from_email_addr(email_addr: EmailAddress) -> Self {
        let endpoint_guid = EndpointGuid(Uuid::from_seed(&email_addr.0));
        Self::new(email_addr, Some(endpoint_guid))
    }

}

impl Display for EndpointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.email_addr)?;

        if let Some(endpoint_guid) = self.endpoint_guid.as_ref() {
            write!(f, ";{}", endpoint_guid)?;
        }

        Ok(())

    }
}

impl FromStr for EndpointId {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(';').collect();
        let email_addr = EmailAddress::from_str(split[0])?;

        let endpoint_guid = if split.len() >= 2 {
            Some(EndpointGuid::from_str(split[1])?)
        } else {
            None
        };

        Ok(EndpointId {email_addr, endpoint_guid})
    }
}
