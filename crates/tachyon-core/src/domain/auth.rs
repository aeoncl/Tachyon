use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct TachyonToken(Arc<str>);

impl TachyonToken {
    pub fn new(token: impl AsRef<str>) -> Self {
        Self(Arc::from(token.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for TachyonToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("TachyonToken(<redacted>)")
    }
}

/// How far a stored login has come: authenticated, then device-trusted, then usable by
/// a bridge. A login only ever moves forward through these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    AuthNeeded,
    VerificationNeeded,
    Ready,
}

impl Readiness {
    /// The single source of truth for which readiness transitions are legal. Staying in
    /// the current state counts as legal so that callers settling an already settled
    /// login do not have to special-case it.
    pub fn can_advance_to(self, next: Readiness) -> bool {
        use Readiness::{AuthNeeded, Ready, VerificationNeeded};
        matches!(
            (self, next),
            (AuthNeeded, AuthNeeded | VerificationNeeded | Ready)
                | (VerificationNeeded, VerificationNeeded | Ready)
                | (Ready, Ready)
        )
    }
}

pub enum RestoreOutcome {
    Success,
    SoftLogout,
    Logout,
}

pub enum InteractiveAuthStarted {
    OAuth {
        auth_url: String,
        csrf_token: String,
    },
    PasswordRequired,
}

pub struct BridgeMetadata {
    pub name: String,
    pub client_uri: String,
    pub image_url: Option<String>,
    pub tos: Option<String>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct CredentialBlob(Vec<u8>);

impl CredentialBlob {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl Debug for CredentialBlob {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("CredentialBlob(<redacted>)")
    }
}

#[cfg(test)]
mod tests {
    use super::Readiness::{AuthNeeded, Ready, VerificationNeeded};

    #[test]
    fn readiness_advances_forward_and_stays_put_but_never_goes_back() {
        let legal = [
            (AuthNeeded, AuthNeeded),
            (AuthNeeded, VerificationNeeded),
            (AuthNeeded, Ready),
            (VerificationNeeded, VerificationNeeded),
            (VerificationNeeded, Ready),
            (Ready, Ready),
        ];
        let illegal = [
            (VerificationNeeded, AuthNeeded),
            (Ready, AuthNeeded),
            (Ready, VerificationNeeded),
        ];

        for (from, to) in legal {
            assert!(from.can_advance_to(to), "{from:?} -> {to:?} must be legal");
        }
        for (from, to) in illegal {
            assert!(!from.can_advance_to(to), "{from:?} -> {to:?} must be refused");
        }

        let all = [AuthNeeded, VerificationNeeded, Ready];
        assert_eq!(
            legal.len() + illegal.len(),
            all.len() * all.len(),
            "every pair of readiness states must be covered"
        );
    }
}
