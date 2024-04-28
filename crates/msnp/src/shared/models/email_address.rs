use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::Arc;
use crate::shared::errors::IdentifierError;

#[derive(Clone, Debug, Eq, Hash, PartialEq, Default)]
pub struct EmailAddress(pub(crate) String);

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
}

impl Into<String> for EmailAddress {
    fn into(self) -> String {
        self.0
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}