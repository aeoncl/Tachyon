use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::anyhow;
use async_trait::async_trait;
use matrix_sdk::authentication::oauth::UrlOrQuery;
use matrix_sdk::{Client, SessionChange};
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;

use tachyon_core::application::error::{BackendError, VerificationError};
use tachyon_core::application::ports::{BackendSession, CredentialRepository};
use tachyon_core::domain::ids::{DeviceId, LoginId};
use tachyon_core::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};

use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::backend::verification::{PendingReset, VerificationFlowMatrix, sas_of};
use crate::infrastructure::mappers::IntoMapper;

pub struct BackendSessionMatrix {
    client: Client,
    login_id: LoginId,
    credential_repository: Arc<dyn CredentialRepository>,
    tasks_token: CancellationToken,
    verification: Mutex<Option<VerificationFlowMatrix>>,
    reset: Mutex<Option<PendingReset>>,
    closed: AtomicBool,
}

impl BackendSessionMatrix {
    pub(crate) fn new(
        client: Client,
        login_id: LoginId,
        credential_repository: Arc<dyn CredentialRepository>,
    ) -> Self {
        Self {
            client,
            login_id,
            credential_repository,
            tasks_token: CancellationToken::new(),
            verification: Mutex::new(None),
            reset: Mutex::new(None),
            closed: AtomicBool::new(false),
        }
    }

    /// FIXME: Remove this after the refactor is done
    pub fn matrix_client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn spawn_token_watcher(&self) {
        let mut receiver = self.client.subscribe_to_session_changes();
        let login_id = self.login_id.clone();
        let client = self.client.clone();
        let credential_repository = self.credential_repository.clone();
        let cancellation_token = self.tasks_token.clone();

        tokio::spawn(async move {
            loop {
                select! {
                    _cancel = cancellation_token.cancelled() => break,

                    session_change = receiver.recv() => {
                        let session_change = match session_change {
                            Ok(session_change) => session_change,
                            Err(RecvError::Lagged(_)) => continue,
                            Err(RecvError::Closed) => break,
                        };

                        match session_change {
                            SessionChange::UnknownToken { soft_logout: _ } => {
                                //Todo push Logout or SoftLogoutEvent
                            }
                            SessionChange::TokensRefreshed => {
                                persist_tokens(&login_id, &client, &credential_repository).await;
                            }
                        }
                    }
                }
            }
        });
    }

    pub(crate) async fn store_credentials(&self) -> Result<(), BackendError> {
        let session = self
            .client
            .session()
            .ok_or_else(|| BackendError::Technical(anyhow!("Client has no session after login")))?;

        let blob = SessionRestoreData::try_from(session)
            .map_err(BackendError::Technical)?
            .to_blob()
            .map_err(BackendError::Technical)?;

        self.credential_repository
            .store(&self.login_id, blob)
            .await?;

        Ok(())
    }

    fn ensure_open(&self) -> Result<(), BackendError> {
        if self.closed.load(Ordering::SeqCst) {
            return Err(BackendError::Technical(anyhow!("session closed")));
        }
        Ok(())
    }

    fn shutdown(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.tasks_token.cancel();
        drop(lock(&self.verification).take());
        drop(lock(&self.reset).take());
    }
}

#[async_trait]
impl BackendSession for BackendSessionMatrix {
    async fn finish_interactive_login(&self, callback_query: &str) -> Result<(), BackendError> {
        self.ensure_open()?;

        self.client
            .oauth()
            .finish_login(UrlOrQuery::Query(callback_query.to_string()))
            .await
            .map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        self.store_credentials().await?;
        self.spawn_token_watcher();

        Ok(())
    }

    async fn device_status(&self) -> Result<DeviceStatus, BackendError> {
        self.ensure_open()?;

        let encryption = self.client.encryption();
        encryption.wait_for_e2ee_initialization_tasks().await;

        let user_id = self
            .client
            .user_id()
            .ok_or_else(|| BackendError::Technical(anyhow!("Client has no user id")))?;

        encryption
            .request_user_identity(user_id)
            .await
            .map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        let own_device = encryption
            .get_own_device()
            .await
            .map_err(|e| BackendError::Technical(anyhow!("{}", e)))?;

        let Some(own_device) = own_device else {
            return Ok(DeviceStatus::Unverified);
        };

        Ok(match own_device.is_cross_signed_by_owner() {
            true => DeviceStatus::Verified,
            false => DeviceStatus::Unverified,
        })
    }

    async fn verification_options(&self) -> Result<VerificationOptions, VerificationError> {
        self.ensure_open()?;
        Err(not_implemented())
    }

    async fn recover(&self, _key: &RecoveryKey) -> Result<DeviceStatus, VerificationError> {
        self.ensure_open()?;
        Err(not_implemented())
    }

    async fn start_device_verification(&self, device: &DeviceId) -> Result<(), VerificationError> {
        self.ensure_open()?;

        let user_id = self
            .client
            .user_id()
            .ok_or_else(|| technical("Client has no user id"))?;
        let Ok(device_id) = device.clone().map_into();

        let device = self
            .client
            .encryption()
            .get_device(user_id, &device_id)
            .await
            .map_err(technical)?
            .ok_or_else(|| technical(format!("unknown device {device_id}")))?;

        let previous = lock(&self.verification).take();
        if let Some(previous) = previous {
            previous.cancel().await;
        }

        let flow =
            VerificationFlowMatrix::start(&self.client, device, self.tasks_token.child_token())
                .await?;

        let raced = lock(&self.verification).replace(flow);
        if let Some(raced) = raced {
            raced.cancel().await;
        }

        Ok(())
    }

    fn verification_state(&self) -> Option<VerificationFlowState> {
        self.ensure_open().ok()?;
        lock(&self.verification)
            .as_ref()
            .map(VerificationFlowMatrix::state)
    }

    async fn verification_action(&self, action: VerificationAction) -> Result<(), VerificationError> {
        self.ensure_open()?;

        let request = lock(&self.verification)
            .as_ref()
            .map(|flow| flow.request().clone())
            .ok_or(VerificationError::NoVerificationInProgress)?;

        match action {
            VerificationAction::Cancel => request.cancel().await.map_err(technical),
            VerificationAction::Confirm => sas_of(&request)
                .ok_or(VerificationError::NoVerificationInProgress)?
                .confirm()
                .await
                .map_err(technical),
            VerificationAction::Mismatch => sas_of(&request)
                .ok_or(VerificationError::NoVerificationInProgress)?
                .mismatch()
                .await
                .map_err(technical),
        }
    }

    async fn reset_identity(
        &self,
        _auth: Option<ResetAuth>,
    ) -> Result<IdentityReset, VerificationError> {
        self.ensure_open()?;
        Err(not_implemented())
    }

    async fn close(&self) {
        self.shutdown();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for BackendSessionMatrix {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn not_implemented() -> VerificationError {
    VerificationError::Backend(BackendError::Technical(anyhow!("not implemented")))
}

fn technical(error: impl std::fmt::Display) -> VerificationError {
    VerificationError::Backend(BackendError::Technical(anyhow!("{error}")))
}

/// A panic in one of the short critical sections must not stop `close` from draining the
/// session, so poisoning is recovered from rather than propagated.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn persist_tokens(
    login_id: &LoginId,
    client: &Client,
    credential_repository: &Arc<dyn CredentialRepository>,
) {
    let Some(session) = client.session() else {
        log::warn!("Tokens refreshed but the client has no session to persist");
        return;
    };

    let blob = SessionRestoreData::try_from(session).and_then(|data| data.to_blob());

    match blob {
        Ok(blob) => {
            if let Err(e) = credential_repository.store(login_id, blob).await {
                log::warn!("Could not persist refreshed tokens: {:?}", e);
            }
        }
        Err(e) => log::warn!("Could not serialize refreshed tokens: {:?}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::AuthKind;
    use crate::infrastructure::backend::auth_service::tests::build_test_client;
    use tachyon_testkit::repositories::CredentialRepositoryInMem;

    fn session(client: Client) -> BackendSessionMatrix {
        BackendSessionMatrix::new(
            client,
            LoginId::new("l1"),
            Arc::new(CredentialRepositoryInMem::default()),
        )
    }

    #[tokio::test]
    async fn close_cancels_the_session_tasks() {
        let (client, _mock) = build_test_client().await;
        let session = session(client);

        session.close().await;

        assert!(session.tasks_token.is_cancelled());
    }

    #[tokio::test]
    async fn dropping_a_session_that_was_never_closed_cancels_the_session_tasks() {
        let (client, _mock) = build_test_client().await;
        let session = session(client);
        let tasks_token = session.tasks_token.clone();

        drop(session);

        assert!(tasks_token.is_cancelled());
    }

    #[tokio::test]
    async fn calls_after_close_are_refused() {
        let (client, _mock) = build_test_client().await;
        let session = session(client);

        session.close().await;

        assert!(matches!(
            session.device_status().await,
            Err(BackendError::Technical(_))
        ));
        assert!(matches!(
            session.recover(&RecoveryKey::new("key")).await,
            Err(VerificationError::Backend(BackendError::Technical(_)))
        ));
        assert!(session.verification_state().is_none());
    }

    #[tokio::test]
    async fn closing_twice_is_allowed() {
        let (client, _mock) = build_test_client().await;
        let session = session(client);

        session.close().await;
        session.close().await;

        assert!(session.tasks_token.is_cancelled());
    }

    #[tokio::test]
    async fn no_verification_state_before_a_flow_starts() {
        let (client, _mock) = build_test_client().await;
        let session = session(client);

        assert!(session.verification_state().is_none());
    }

    #[tokio::test]
    async fn an_action_without_a_flow_is_refused() {
        let (client, _mock) = build_test_client().await;
        let session = session(client);

        assert!(matches!(
            session.verification_action(VerificationAction::Confirm).await,
            Err(VerificationError::NoVerificationInProgress)
        ));
    }

    #[tokio::test]
    async fn starting_a_verification_without_a_logged_in_client_fails() {
        let (client, _mock) = build_test_client().await;
        let session = session(client);

        assert!(matches!(
            session.start_device_verification(&DeviceId::new("OTHER")).await,
            Err(VerificationError::Backend(BackendError::Technical(_)))
        ));
        assert!(session.verification_state().is_none());
    }

    #[tokio::test]
    async fn stored_credentials_round_trip_through_the_repository() {
        let (client, _mock) = build_test_client().await;
        let restore_data = SessionRestoreData {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            user_id: matrix_sdk::ruma::UserId::parse("@aeon:shlasouf.local")
                .unwrap()
                .to_owned(),
            device_id: matrix_sdk::ruma::OwnedDeviceId::from("DEVICEID"),
            auth_kind: AuthKind::Matrix,
        };
        client.restore_session(restore_data.clone()).await.unwrap();

        let repository = Arc::new(CredentialRepositoryInMem::default());
        let login_id = LoginId::new("l1");
        let session =
            BackendSessionMatrix::new(client, login_id.clone(), repository.clone());

        session.store_credentials().await.unwrap();

        let blob = repository.credentials(&login_id).await.unwrap().unwrap();
        let restored = SessionRestoreData::from_blob(&blob).unwrap();
        assert!(restored == restore_data);
    }
}
