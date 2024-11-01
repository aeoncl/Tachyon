use std::str::FromStr;

use anyhow::anyhow;
use crate::msnp::error::CommandError;

#[derive(Debug, Clone)]
pub struct TicketToken(pub String);

impl TicketToken {

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl std::fmt::Display for TicketToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "t={}", self.0)
    }
}

impl FromStr  for TicketToken {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let no_prefix = s.strip_prefix("t=").ok_or(Self::Err::ArgumentParseError { argument: s.to_string(), command: String::new(), source: anyhow!("Error stripping t= prefix from Ticket Token")})?;
        Ok(Self(no_prefix.to_string()))
    }
}



impl PartialEq<str> for TicketToken {
    fn eq(&self, other: &str) -> bool {
        &self.0 == other
    }

    fn ne(&self, other: &str) -> bool {
        &self.0 != other
    }
}

impl PartialEq<String> for TicketToken {
    fn eq(&self, other: &String) -> bool {
        &self.0 == other
    }

    fn ne(&self, other: &String) -> bool {
        &self.0 != other
    }
}

