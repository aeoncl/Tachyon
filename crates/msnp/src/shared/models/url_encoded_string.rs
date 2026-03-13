use crate::msnp::error::CommandError;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Eq, PartialEq, Debug, Clone, Default)]
pub struct UrlEncodedString(String);

impl UrlEncodedString {
    pub fn new(value: String) -> Self {
      Self(value)
    }

    pub fn new_from_ref(value: &str) -> Self {
        Self(value.to_owned())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

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