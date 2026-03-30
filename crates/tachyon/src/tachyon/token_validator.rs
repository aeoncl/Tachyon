use hmac::digest::KeyInit;
use hmac::{Hmac, Mac};
use msnp::shared::models::email_address::EmailAddress;
use sha2::Sha256;
use std::cmp::PartialEq;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::OnceLock;
use msnp::shared::models::ticket_token::TicketToken;

type HmacSha256 = Hmac<Sha256>;

pub struct TokenValidator {
    secret: Vec<u8>,
}

impl TokenValidator {
    pub fn new(secret: &str) -> Result<Self, anyhow::Error> {
        if secret.is_empty() {
            return Err(anyhow::anyhow!("Secret cannot be empty"));
        }
        HmacSha256::new_from_slice(secret.as_bytes())?; // fail fast validation
        Ok(Self { secret: secret.as_bytes().to_vec() })
    }

    pub fn generate(&self, email: &EmailAddress) -> TicketToken {
        let mut mac = HmacSha256::new_from_slice(&self.secret).expect("HMAC initialization to never fail at this point.");
        mac.update(email.as_bytes());
        TicketToken(hex::encode(mac.finalize().into_bytes()))
    }

    pub fn validate(&self, email: &EmailAddress, candidate: &str) -> bool {
        let expected = self.generate(email);
        &expected == candidate
    }
}

#[cfg(test)]
mod tests {
    use crate::tachyon::token_validator::TokenValidator;
    use msnp::shared::models::email_address::EmailAddress;
    use std::str::FromStr;

    const TEST_SECRET: &str = "a_very_secret_key_for_testing";
    const TEST_EMAIL: &str = "localpart@domain.com";

    #[test]
    fn new_succeeds_with_valid_secret() {
        assert!(TokenValidator::new(TEST_SECRET).is_ok());
    }

    #[test]
    fn new_fails_with_empty_secret() {
        assert!(TokenValidator::new("").is_err());
    }

    #[test]
    fn generate_returns_token() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let token = validator.generate(&EmailAddress::from_str(TEST_EMAIL).unwrap());
        assert!(!token.as_str().is_empty());
    }

    #[test]
    fn generate_is_deterministic() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let email = EmailAddress::from_str(TEST_EMAIL).unwrap();
        let token1 = validator.generate(&email);
        let token2 = validator.generate(&email);
        assert_eq!(token1, token2);
    }

    #[test]
    fn generate_differs_for_different_emails() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let token1 = validator.generate(&EmailAddress::from_str("alice@domain.com").unwrap());
        let token2 = validator.generate(&EmailAddress::from_str("bob@domain.com").unwrap());
        assert_ne!(token1, token2);
    }

    #[test]
    fn generate_differs_for_different_secrets() {
        let validator1 = TokenValidator::new("secret_one").unwrap();
        let validator2 = TokenValidator::new("secret_two").unwrap();
        let email = EmailAddress::from_str(TEST_EMAIL).unwrap();
        assert_ne!(validator1.generate(&email), validator2.generate(&email));
    }

    #[test]
    fn generate_produces_64_char_hex_string() {
        let token = TokenValidator::new(TEST_SECRET).unwrap().generate(&EmailAddress::from_str(TEST_EMAIL).unwrap());
        assert_eq!(token.as_str().len(), 64);
        assert!(token.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn validate_succeeds_with_correct_token() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let email = EmailAddress::from_str(TEST_EMAIL).unwrap();
        let token = validator.generate(&email);
        assert!(validator.validate(&email, token.as_str()));
    }

    #[test]
    fn validate_fails_with_wrong_token() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        assert!(!validator.validate(&EmailAddress::from_str(TEST_EMAIL).unwrap(), "wrong_token"));
    }

    #[test]
    fn validate_fails_with_empty_token() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        assert!(!validator.validate(&EmailAddress::from_str(TEST_EMAIL).unwrap(), ""));
    }

    #[test]
    fn validate_fails_with_different_email() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let token = validator.generate(&EmailAddress::from_str(TEST_EMAIL).unwrap());
        let other_email = EmailAddress::from_str("other@domain.com").unwrap();
        assert!(!validator.validate(&other_email, token.as_str()));
    }

    #[test]
    fn validate_fails_with_token_from_different_secret() {
        let validator1 = TokenValidator::new("secret_one").unwrap();
        let validator2 = TokenValidator::new("secret_two").unwrap();
        let email = EmailAddress::from_str(TEST_EMAIL).unwrap();
        let token = validator1.generate(&email);
        assert!(!validator2.validate(&email, token.as_str()));
    }

    #[test]
    fn token_eq_str() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let token = validator.generate(&EmailAddress::from_str(TEST_EMAIL).unwrap());
        assert!(token == token.as_str());
    }

    #[test]
    fn token_eq_string() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let token = validator.generate(&EmailAddress::from_str(TEST_EMAIL).unwrap());
        let as_string = token.as_str().to_string();
        assert_eq!(token, as_string);
    }

    #[test]
    fn token_ne_different_str() {
        let validator = TokenValidator::new(TEST_SECRET).unwrap();
        let token = validator.generate(&EmailAddress::from_str(TEST_EMAIL).unwrap());
        assert!(!(token == "not_the_right_token"));
    }
}