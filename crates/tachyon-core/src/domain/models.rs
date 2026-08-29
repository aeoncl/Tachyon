use std::str::FromStr;
use crate::domain::error::TachyonError;
use crate::domain::ids::UserId;

pub struct Participant {
    id: UserId

}

pub struct EmailAddress(String);

impl EmailAddress {

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn crack(&self) -> (&str, &str) {
        self.0.split_once("@").expect("To be a valid email")
    }

}

impl FromStr for EmailAddress {
    type Err = TachyonError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let indices: Vec<_> = s.match_indices("@").collect();
        if indices.len() == 1 {
            Ok(EmailAddress(s.into()))
        } else {
            Err(TachyonError::InvalidEmail(s.to_string()))
        }
    }
}