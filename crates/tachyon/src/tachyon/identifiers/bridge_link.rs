use std::fmt::{Display, Formatter};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeType {
    Msn,
}

impl Display for BridgeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeType::Msn => write!(f, "msn"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClientVersion(pub u32);

pub const DEFAULT_CLIENT_VERSION: ClientVersion = ClientVersion(14);

impl ClientVersion {
    pub fn from_user_agent(user_agent: &str) -> Self {
        let mut any_product_major = None;
        for token in user_agent.split_whitespace() {
            let (product, version) = match token.split_once('/') {
                Some(parts) => parts,
                None => continue,
            };
            let Some(major) = major_of(version) else {
                continue;
            };
            let product = product.to_ascii_lowercase();
            if product.contains("msn") || product.contains("messenger") {
                return ClientVersion(major);
            }
            any_product_major = any_product_major.or(Some(major));
        }
        any_product_major
            .or_else(|| major_at_start(user_agent))
            .map_or(DEFAULT_CLIENT_VERSION, ClientVersion)
    }
}

fn major_of(version: &str) -> Option<u32> {
    major_at_start(version).filter(|_| version.contains('.'))
}

fn major_at_start(value: &str) -> Option<u32> {
    let major: String = value.chars().take_while(|c| c.is_ascii_digit()).collect();
    if major.is_empty() {
        return None;
    }
    major.parse().ok()
}

// never decoded: anything needing the user id looks it up next to the token
// not authenticated: any local process can fabricate one; Tachyon only serves localhost
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BridgeLinkToken(String);

impl BridgeLinkToken {
    pub fn from_parts(bridge: BridgeType, user_id: &str, version: ClientVersion) -> Self {
        let canonical = format!("{}|{}|{}", bridge, user_id.to_ascii_lowercase(), version.0);
        let digest = Sha256::digest(canonical.as_bytes());
        Self(hex(&digest))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for BridgeLinkToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for BridgeLinkToken {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err("token must not be empty")
        } else {
            Ok(Self(s.to_string()))
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn token_is_deterministic_for_the_same_triple() {
        let a = BridgeLinkToken::from_parts(BridgeType::Msn, "User@Example.org", ClientVersion(14));
        let b = BridgeLinkToken::from_parts(BridgeType::Msn, "user@example.org", ClientVersion(14));
        assert_eq!(a, b, "user_id is canonicalized to lowercase");
    }

    #[test]
    fn token_differs_when_any_part_differs() {
        let base = BridgeLinkToken::from_parts(BridgeType::Msn, "user@example.org", ClientVersion(14));
        assert_ne!(base, BridgeLinkToken::from_parts(BridgeType::Msn, "other@example.org", ClientVersion(14)));
        assert_ne!(base, BridgeLinkToken::from_parts(BridgeType::Msn, "user@example.org", ClientVersion(15)));
    }

    #[test]
    fn token_is_hex_and_not_the_raw_triple() {
        let token = BridgeLinkToken::from_parts(BridgeType::Msn, "user@example.org", ClientVersion(14));
        assert!(token.as_str().chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(token.as_str().len(), 64);
        assert!(!token.as_str().contains("user@example.org"));
    }

    #[test]
    fn token_round_trips_through_str() {
        let token = BridgeLinkToken::from_parts(BridgeType::Msn, "user@example.org", ClientVersion(14));
        let parsed: BridgeLinkToken = token.as_str().parse().expect("valid hex token");
        assert_eq!(token, parsed);
        assert!("".parse::<BridgeLinkToken>().is_err());
    }

    #[test]
    fn version_comes_from_the_messenger_product_token() {
        assert_eq!(
            ClientVersion::from_user_agent("MSNMSGR/14.0.8064.0205 msmsgs"),
            ClientVersion(14)
        );
        assert_eq!(
            ClientVersion::from_user_agent("Windows-Messenger/8.5 other/2.1"),
            ClientVersion(8)
        );
        assert_eq!(ClientVersion::from_user_agent("no versions here"), DEFAULT_CLIENT_VERSION);
    }
}
