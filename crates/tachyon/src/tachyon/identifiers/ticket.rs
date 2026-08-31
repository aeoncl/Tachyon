use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use sha1::{Digest, Sha1};
use tachyon_core::domain::auth::TachyonToken;

/// Derives the ticket the MSN client stores and echoes back.
///
/// The client calls RST2 on every startup with the password it saved, and that password is
/// meaningless to us — the ticket's only job is to name an account so a backend session can
/// be restored. So it is derived rather than issued: RST2 and the `USR` handler compute the
/// same value for the same address with no state between them, across restarts.
///
/// Keyed on the instance's `local.key` so the value is not derivable off-box by anyone who
/// merely knows the address. That is the whole of its protection — a ticket is bearer
/// authority over the account, which is only tolerable because Tachyon is bound to loopback
/// and serves a single client.
pub fn derive_ticket(secret: &[u8], email: &EmailAddress) -> TicketToken {
    let mut hasher = Sha1::new();
    hasher.update(secret);
    hasher.update(b":");
    hasher.update(email.as_str().to_lowercase().as_bytes());

    TicketToken(hex::encode(hasher.finalize()))
}

/// The same value as core sees it. Core treats it as opaque.
pub fn derive_token(secret: &[u8], email: &EmailAddress) -> TachyonToken {
    TachyonToken::new(derive_ticket(secret, email).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    const TEST_SECRET: [u8; 4] = [1, 2, 3, 4];

    fn email(raw: &str) -> EmailAddress {
        EmailAddress::from_str(raw).unwrap()
    }

    #[test]
    fn derivation_is_stable_for_the_same_address() {
        let first = derive_ticket(&TEST_SECRET, &email("aeon@shlasouf.local"));
        let second = derive_ticket(&TEST_SECRET, &email("aeon@shlasouf.local"));

        assert_eq!(first, second, "the client must get the same ticket every restart");
    }

    #[test]
    fn derivation_ignores_address_case() {
        let lower = derive_ticket(&TEST_SECRET, &email("aeon@shlasouf.local"));
        let upper = derive_ticket(&TEST_SECRET, &email("AEON@shlasouf.local"));

        assert_eq!(lower, upper);
    }

    #[test]
    fn different_addresses_derive_different_tickets() {
        let one = derive_ticket(&TEST_SECRET, &email("aeon@shlasouf.local"));
        let other = derive_ticket(&TEST_SECRET, &email("someone@shlasouf.local"));

        assert_ne!(one, other);
    }

    #[test]
    fn different_secrets_derive_different_tickets() {
        let one = derive_ticket(&TEST_SECRET, &email("aeon@shlasouf.local"));
        let other = derive_ticket(&[9, 9, 9, 9], &email("aeon@shlasouf.local"));

        assert_ne!(one, other, "the ticket must not be derivable without local.key");
    }
}
