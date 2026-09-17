use crate::domain::ids::{BridgeId, ClientVersion, UserId};
use crate::domain::verification::Password;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use uuid::Uuid;

const BRIDGE_LINK_NAMESPACE: Uuid = Uuid::from_u128(0x7ac4e0b1_5a2d_4d5e_9c3f_2b1a0f9e8d7c);

#[derive(Clone, Hash, Eq, PartialEq)]
pub struct BridgeLinkToken(Arc<str>);

impl BridgeLinkToken {

    pub fn mint(bridge: &BridgeId, user: &UserId, client: &ClientVersion) -> Self {
        let name = format!("{bridge}:{}:{client}", user.as_str().to_lowercase());
        Self::new(
            Uuid::new_v5(&BRIDGE_LINK_NAMESPACE, name.as_bytes())
                .simple()
                .to_string(),
        )
    }

    pub fn new(token: impl AsRef<str>) -> Self {
        Self(Arc::from(token.as_ref()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for BridgeLinkToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("BridgeLinkToken(<redacted>)")
    }
}

#[derive(Clone, Debug)]
pub enum InteractiveAuthStarted {
    OAuth {
        auth_url: String,
        csrf_token: String,
    },
    PasswordRequired,
}

/// What completes an interactive login, matching the prompt it started with.
#[derive(Debug)]
pub enum Credential {
    /// The raw query string the authorization server redirected the browser back with.
    OAuthCallback(String),
    Password(Password),
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

/// What ending a login does to its session and its store row. Nothing else in core touches
/// either. `LogOut` is the only effect that ends a device.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Effect {
    /// Close the session, keeping what the backend has on disk so a later sign-in restores it.
    Close,
    /// Close the session and drop what the backend keeps on disk for it.
    Discard,
    /// End the device on the backend.
    LogOut,
    /// Drop the login's store row, so no token points at it any more.
    DeleteRow,
}

/// Why a live login is ending.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ending {
    /// The client went away, or the authorization server refused the flow. Before the backend
    /// accepted a credential there is no device and nothing worth restoring.
    Dropped { authenticated: bool },
    /// It authenticated but could not be settled while the token still held it. The row
    /// stays: the next sign-in restores it.
    Unsettled,
    /// It authenticated after the token stopped holding it. Nothing points at the device it
    /// just made.
    Orphaned,
    /// The user asked for the login to go, for good.
    Deleted,
}

/// The effects an ending runs, in order. `LogOut` precedes `Discard` so the device ends
/// before its on-disk state goes.
pub fn ending(ending: Ending) -> &'static [Effect] {
    use Effect::*;
    match ending {
        Ending::Dropped { authenticated: false } => &[Discard, DeleteRow],
        Ending::Dropped { authenticated: true } | Ending::Unsettled => &[Close],
        Ending::Orphaned => &[LogOut, Discard],
        Ending::Deleted => &[LogOut, Discard, DeleteRow],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msn() -> BridgeId {
        BridgeId::new("msn")
    }

    fn v14() -> ClientVersion {
        ClientVersion::new("14.0")
    }

    fn aeon() -> UserId {
        UserId::new("@aeon:shlasouf.local")
    }

    #[test]
    fn the_same_inputs_mint_the_same_token() {
        assert_eq!(
            BridgeLinkToken::mint(&msn(), &aeon(), &v14()),
            BridgeLinkToken::mint(&msn(), &aeon(), &v14()),
            "the client must get the same ticket every sign-in"
        );
    }

    #[test]
    fn two_bridges_mint_different_tokens_for_one_user() {
        assert_ne!(
            BridgeLinkToken::mint(&msn(), &aeon(), &v14()),
            BridgeLinkToken::mint(&BridgeId::new("yahoo"), &aeon(), &v14())
        );
    }

    #[test]
    fn two_users_mint_different_tokens_on_one_bridge() {
        assert_ne!(
            BridgeLinkToken::mint(&msn(), &aeon(), &v14()),
            BridgeLinkToken::mint(&msn(), &UserId::new("@someone:shlasouf.local"), &v14())
        );
    }

    #[test]
    fn two_client_versions_mint_different_tokens() {
        assert_ne!(
            BridgeLinkToken::mint(&msn(), &aeon(), &v14()),
            BridgeLinkToken::mint(&msn(), &aeon(), &ClientVersion::new("8.5"))
        );
    }

    #[test]
    fn the_user_id_case_does_not_change_the_token() {
        assert_eq!(
            BridgeLinkToken::mint(&msn(), &aeon(), &v14()),
            BridgeLinkToken::mint(&msn(), &UserId::new("@AEON:shlasouf.local"), &v14())
        );
    }

    #[test]
    fn a_login_dropped_before_it_authenticated_leaves_nothing_behind() {
        assert_eq!(
            ending(Ending::Dropped { authenticated: false }),
            [Effect::Discard, Effect::DeleteRow]
        );
    }

    #[test]
    fn only_an_orphan_and_a_deletion_end_the_device() {
        for why in [
            Ending::Dropped { authenticated: true },
            Ending::Dropped { authenticated: false },
            Ending::Unsettled,
        ] {
            assert!(!ending(why).contains(&Effect::LogOut), "{why:?} must not end the device");
        }
        assert!(ending(Ending::Orphaned).contains(&Effect::LogOut));
        assert!(ending(Ending::Deleted).contains(&Effect::LogOut));
    }

    #[test]
    fn a_login_that_authenticated_keeps_its_row_unless_the_user_deleted_it() {
        assert_eq!(ending(Ending::Unsettled), [Effect::Close]);
        assert_eq!(ending(Ending::Dropped { authenticated: true }), [Effect::Close]);
        assert_eq!(
            ending(Ending::Deleted),
            [Effect::LogOut, Effect::Discard, Effect::DeleteRow]
        );
    }
}
