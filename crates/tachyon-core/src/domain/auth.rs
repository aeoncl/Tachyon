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
