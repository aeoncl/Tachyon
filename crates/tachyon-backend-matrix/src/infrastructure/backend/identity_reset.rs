use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use futures_util::FutureExt;
use matrix_sdk::Client;
use matrix_sdk::encryption::CrossSigningResetAuthType;
use matrix_sdk::encryption::recovery::IdentityResetHandle;
use matrix_sdk::ruma::UserId;
use matrix_sdk::ruma::api::client::uiaa::{AuthData, Password as PasswordAuth, UserIdentifier};
use tokio::task::JoinHandle;

use tachyon_core::domain::verification::{IdentityReset, Password, RecoveryKey};

/// The SDK retries the signing-key upload with no sleep until the homeserver accepts it, so a
/// reset waiting for an approval that never comes has to be cut off.
const APPROVAL_WINDOW: Duration = Duration::from_secs(3 * 60);

/// One `recovery().reset_identity()` per user flow. That call deletes the key backup and
/// disables secret storage before it reports which auth the homeserver wants, so the slot is
/// claimed as `Starting` before the call and a second caller only ever observes it.
pub(crate) enum PendingReset {
    Starting,
    PasswordRequired {
        handle: Arc<IdentityResetHandle>,
        uiaa_session: Option<String>,
    },
    ApprovalRequired {
        handle: Arc<IdentityResetHandle>,
        url: String,
        running: Option<JoinHandle<anyhow::Result<RecoveryKey>>>,
    },
    Done {
        recovery_key: RecoveryKey,
    },
}

impl PendingReset {
    fn from_handle(handle: IdentityResetHandle) -> Self {
        match handle.auth_type() {
            CrossSigningResetAuthType::Uiaa(info) => Self::PasswordRequired {
                uiaa_session: info.session.clone(),
                handle: Arc::new(handle),
            },
            CrossSigningResetAuthType::OAuth(info) => Self::ApprovalRequired {
                url: info.approval_url.to_string(),
                handle: Arc::new(handle),
                running: None,
            },
        }
    }

    /// What the caller should be told now. A reset task that has finished is folded in first:
    /// its key becomes `Done`, its failure is returned once and the slot goes back to waiting
    /// for approval.
    pub(crate) fn observe(&mut self) -> anyhow::Result<IdentityReset> {
        self.fold_finished_task()?;

        Ok(match self {
            Self::Starting => IdentityReset::ApprovalPending,
            Self::PasswordRequired { .. } => IdentityReset::PasswordRequired,
            Self::ApprovalRequired {
                running: Some(_), ..
            } => IdentityReset::ApprovalPending,
            Self::ApprovalRequired {
                running: None, url, ..
            } => IdentityReset::ApprovalRequired { url: url.clone() },
            Self::Done { recovery_key } => IdentityReset::Done {
                recovery_key: recovery_key.clone(),
            },
        })
    }

    /// The user says they approved the reset: run it in the background unless one is already
    /// running, and report.
    pub(crate) fn approve(&mut self, client: &Client) -> anyhow::Result<IdentityReset> {
        self.fold_finished_task()?;

        if let Self::ApprovalRequired {
            handle,
            running: running @ None,
            ..
        } = self
        {
            *running = Some(tokio::spawn(approved_reset(client.clone(), handle.clone())));
        }

        self.observe()
    }

    fn fold_finished_task(&mut self) -> anyhow::Result<()> {
        let Self::ApprovalRequired { running, .. } = self else {
            return Ok(());
        };
        let Some(task) = running.as_mut() else {
            return Ok(());
        };
        let Some(outcome) = task.now_or_never() else {
            return Ok(());
        };
        *running = None;

        match outcome {
            Ok(Ok(recovery_key)) => {
                *self = Self::Done { recovery_key };
                Ok(())
            }
            Ok(Err(error)) => Err(error),
            Err(join_error) => Err(anyhow!("the identity reset task stopped: {join_error}")),
        }
    }
}

impl Drop for PendingReset {
    fn drop(&mut self) {
        if let Self::ApprovalRequired {
            running: Some(task),
            ..
        } = self
        {
            task.abort();
        }
    }
}

pub(crate) async fn start_identity_reset(client: &Client) -> anyhow::Result<PendingReset> {
    match client.encryption().recovery().reset_identity().await? {
        Some(handle) => Ok(PendingReset::from_handle(handle)),
        None => Ok(PendingReset::Done {
            recovery_key: enable_recovery(client).await?,
        }),
    }
}

pub(crate) fn password_auth(
    user_id: &UserId,
    uiaa_session: Option<String>,
    password: &Password,
) -> AuthData {
    let mut auth = PasswordAuth::new(
        UserIdentifier::UserIdOrLocalpart(user_id.to_string()),
        password.as_str().to_owned(),
    );
    auth.session = uiaa_session;
    AuthData::Password(auth)
}

pub(crate) async fn run_reset(
    client: &Client,
    handle: &IdentityResetHandle,
    auth: Option<AuthData>,
) -> anyhow::Result<RecoveryKey> {
    handle.reset(auth).await?;
    enable_recovery(client).await
}

async fn approved_reset(
    client: Client,
    handle: Arc<IdentityResetHandle>,
) -> anyhow::Result<RecoveryKey> {
    match tokio::time::timeout(APPROVAL_WINDOW, run_reset(&client, &handle, None)).await {
        Ok(outcome) => outcome,
        Err(_) => Err(anyhow!(
            "the homeserver did not accept the reset within {} seconds; approve it and try again",
            APPROVAL_WINDOW.as_secs()
        )),
    }
}

async fn enable_recovery(client: &Client) -> anyhow::Result<RecoveryKey> {
    let recovery_key = client.encryption().recovery().enable().await?;
    Ok(RecoveryKey::new(recovery_key))
}
