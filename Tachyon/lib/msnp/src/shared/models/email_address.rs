use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub struct EmailAddress(pub(crate) String);

impl FromStr for EmailAddress {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let indices: Vec<_> = s.match_indices("@").collect();
        if indices.len() == 1 {
            Ok(EmailAddress(s.to_string()))
        } else {
            //FU
            todo!()
        }

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