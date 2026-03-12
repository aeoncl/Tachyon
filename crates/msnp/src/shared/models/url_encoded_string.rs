use std::cmp::PartialEq;
use crate::msnp::error::CommandError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use num_derive::FromPrimitive;

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct UrlEncodedString(pub(crate) String);

impl FromStr for UrlEncodedString {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decoded = urlencoding::decode(s)
            .map_err(|e| Self::Err::ArgumentParseError {
                argument: s.to_string(),
                command: String::new(),
                source: e.into(),
        })?.to_string();

        Ok(Self(decoded))
    }
}

impl Display for UrlEncodedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let encoded = urlencoding::encode(&self.0);
        write!(f, "{}", encoded)
    }
}

impl PartialEq<str> for UrlEncodedString {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}