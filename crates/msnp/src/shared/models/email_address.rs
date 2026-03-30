use crate::shared::errors::IdentifierError;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::shared::models::uuid::Uuid;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Default)]
pub struct EmailAddress(String);

impl FromStr for EmailAddress {
    type Err = IdentifierError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let indices: Vec<_> = s.match_indices("@").collect();
        if indices.len() == 1 {
            Ok(EmailAddress(s.into()))
        } else {
            Err(IdentifierError::InvalidEmailAddress(s.to_string()))
        }
    }
}

impl EmailAddress {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn crack(&self) -> (&str, &str) {
        self.0.split_once("@").expect("To be a valid email")
    }
}

impl Into<String> for EmailAddress {
    fn into(self) -> String {
        self.0
    }
}

impl From<EmailAddress> for Uuid {
    fn from(value: EmailAddress) -> Self {
        Uuid::from_seed(value.as_str())
    }
}

impl From<&EmailAddress> for Uuid {
    fn from(value: &EmailAddress) -> Self {
        Uuid::from_seed(value.as_str())
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}
