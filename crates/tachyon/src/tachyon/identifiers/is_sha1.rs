use lazy_static::lazy_static;
use regex::Regex;
use msnp::shared::models::email_address::EmailAddress;

pub trait IsSha1 {
    fn is_sha1_imprecise(&self) -> bool;
}

lazy_static! {
    static ref SHA1_REGEX: Regex = Regex::new("^[0-9a-f]{40}$").unwrap();
}

impl IsSha1 for EmailAddress {
    fn is_sha1_imprecise(&self) -> bool {
        let (local_part, _domain) = self.crack();
        local_part.len() == 40 && SHA1_REGEX.is_match(local_part)
    }
}