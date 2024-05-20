use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::anyhow;
use num_traits::FromPrimitive;

use crate::msnp::error::CommandError;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::network_id::NetworkId;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::network_id::NetworkId;
    use crate::shared::models::network_id_email::NetworkIdEmail;

    #[test]
    fn network_id_ser() {
        let network_id = NetworkIdEmail {
            network_id: NetworkId::WindowsLive,
            email: EmailAddress::from_str("aeon@test.com").unwrap(),
        };

        let ser = network_id.to_string();

        assert_eq!("1:aeon@test.com", &ser);
    }

    #[test]
    fn network_id_deser() {
        let raw = "1:aeon@test.com";

        let deser = NetworkIdEmail::from_str(raw).unwrap();

        assert!(matches!(deser.network_id, NetworkId::WindowsLive));
        assert_eq!("aeon@test.com", deser.email.as_str());
    }
}

#[derive(Debug, Clone)]
pub struct NetworkIdEmail {
    pub network_id: NetworkId,
    pub email: EmailAddress
}

impl NetworkIdEmail {
    pub fn new(network_id: NetworkId, email_address: EmailAddress) -> Self{
        Self {
            network_id,
            email: email_address,
        }
    }
}

impl Display for NetworkIdEmail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.network_id.clone() as i32, self.email)
    }
}

impl FromStr for NetworkIdEmail {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split: Vec<&str> = s.split(':').collect();
        if split.len() != 2 {
            return Err(CommandError::ArgumentParseError {
                argument: "NetworkIdEmail".to_string(),
                command: "".to_string(),
                source: anyhow!("Split length wasnt two {}", s),
            });
        }

        let raw_network_id = u32::from_str(split.swap_remove(0))?;
        let network_id = NetworkId::from_u32(raw_network_id).ok_or(anyhow!("Could not cast network id {}", raw_network_id))?;

        let email_addr = EmailAddress::from_str(split.swap_remove(0))?;

        Ok(NetworkIdEmail {
            network_id,
            email: email_addr,
        })
    }
}