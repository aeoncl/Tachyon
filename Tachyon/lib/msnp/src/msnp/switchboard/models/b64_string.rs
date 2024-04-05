use std::fmt::{Display, Formatter};
use std::str::FromStr;
use base64::Engine;
use base64::engine::general_purpose;
use crate::msnp::error::CommandError;

pub struct Base64String(String);

impl FromStr for Base64String {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw_decoded = general_purpose::STANDARD.decode(s).map_err(|e| Self::Err::ArgumentParseError {
            argument: s.to_string(),
            command: String::new(),
            source: e.into(),
        })?;
        Ok(Base64String(String::from_utf8(raw_decoded)?))
    }
}

impl Display for Base64String {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = general_purpose::STANDARD.encode(self.0.as_bytes());
        write!(f, "{}", encoded)
    }
}