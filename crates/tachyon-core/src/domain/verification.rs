use crate::domain::ids::DeviceId;
use std::fmt::{Debug, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Verified,
    Unverified,
}

#[derive(Debug, Clone)]
pub struct VerificationOptions {
    pub recovery_available: bool,
    pub devices: Vec<DeviceSummary>,
}

#[derive(Debug, Clone)]
pub struct DeviceSummary {
    pub id: DeviceId,
    pub display_name: Option<String>,
}

/// A recovery key or passphrase. Never logged: `Debug` is redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct RecoveryKey(String);

impl RecoveryKey {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Debug for RecoveryKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("RecoveryKey(<redacted>)")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationFlowState {
    /// Sent to the other device; waiting for it to accept.
    Requested,
    /// The other device accepted; keys are being exchanged.
    Started,
    CompareEmojis { emojis: Vec<SasEmoji> },
    /// This side confirmed; waiting for the other device to confirm.
    AwaitingOtherConfirmation,
    Done,
    Cancelled { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SasEmoji {
    pub symbol: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationAction {
    Confirm,
    Mismatch,
    Cancel,
}

#[derive(Debug)]
pub enum IdentityReset {
    /// Identity reset and recovery re-enabled. The key is shown to the user once.
    Done { recovery_key: RecoveryKey },
    /// The homeserver wants the account password; call again with it.
    PasswordRequired,
    /// The homeserver wants the user to approve the reset at `url`; call again once they have.
    ApprovalRequired { url: String },
}
