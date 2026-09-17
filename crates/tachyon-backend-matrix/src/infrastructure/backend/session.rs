use std::any::Any;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use async_trait::async_trait;
use matrix_sdk::utils::UrlOrQuery;
use matrix_sdk::encryption::recovery::{IdentityResetHandle, RecoveryError};
use matrix_sdk::encryption::secret_storage::{ImportError, SecretStorageError};

use matrix_sdk::encryption::VerificationState;
use matrix_sdk::ruma::OwnedUserId;
use matrix_sdk::{Client, SessionChange};
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio_util::sync::CancellationToken;

use tachyon_core::application::error::{BackendError, VerificationError};
use tachyon_core::application::ports::{BackendSession, CredentialRepository};
use tachyon_core::domain::auth::Credential;
use tachyon_core::domain::ids::{DeviceId, LoginId};
use tachyon_core::domain::verification::{
    DeviceStatus, DeviceSummary, IdentityReset, Password, RecoveryKey, ResetAuth,
    VerificationAction, VerificationFlowState, VerificationOptions,
};

use crate::domain::auth::SessionRestoreData;
use crate::infrastructure::backend::auth_service::remove_login_dir;
use crate::infrastructure::backend::identity_reset::{
    PendingReset, password_auth, run_reset, start_identity_reset,
};
use crate::infrastructure::backend::technical;
use crate::infrastructure::backend::verification::{VerificationFlowMatrix, run_to_device_sync, sas_of};
use crate::infrastructure::mappers::IntoMapper;

/// How long `log_out` waits for the homeserver to acknowledge before giving up. The caller
/// is forgetting the login locally either way.
const LOGOUT_WINDOW: Duration = Duration::from_secs(5);

pub struct BackendSessionMatrix {
    client: Client,
    login_id: LoginId,
    /// Who is signing in, for a password login before the client knows it.
    user_id: OwnedUserId,
    /// This login's directory under the store root, if the client persists anything.
    login_dir: Option<PathBuf>,
    discard_store: AtomicBool,
    credential_repository: Arc<dyn CredentialRepository>,
    tasks_token: CancellationToken,
    /// Ends the `sync_until_verified` task. A child of `tasks_token`, so closing ends it too.
    pre_ready_sync: CancellationToken,
    verification: Mutex<Option<VerificationFlowMatrix>>,
    reset: Mutex<Option<PendingReset>>,
}

impl BackendSessionMatrix {
    pub(crate) fn new(
        client: Client,
        login_id: LoginId,
        credential_repository: Arc<dyn CredentialRepository>,
        user_id: OwnedUserId,
        login_dir: Option<PathBuf>,
    ) -> Self {
        let tasks_token = CancellationToken::new();
        Self {
            client,
            login_id,
            user_id,
            login_dir,
            discard_store: AtomicBool::new(false),
            credential_repository,
            pre_ready_sync: tasks_token.child_token(),
            tasks_token,
            verification: Mutex::new(None),
            reset: Mutex::new(None),
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
                            SessionChange::UnknownToken(_) => {
                                //Todo push Logout or SoftLogoutEvent
                            }
                            SessionChange::TokensRefreshed => {
                                if let Err(e) = store_session(&login_id, &client, &credential_repository).await {
                                    log::warn!("Could not persist refreshed tokens: {e:?}");
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    /// Syncs to-device traffic until this device is trusted. A fresh device's keys only reach
    /// the homeserver through a sync's outgoing requests, and the signatures other devices put
    /// on them only come back through one, so without this no verification method can finish.
    /// Ends on its own once the device reads as verified, and with the session.
    pub(crate) fn sync_until_verified(&self) {
        let client = self.client.clone();
        let cancel = self.pre_ready_sync.clone();

        tokio::spawn(async move {
            client.encryption().wait_for_e2ee_initialization_tasks().await;
            let mut states = client.encryption().verification_state();
            let verified = async {
                while let Some(state) = states.next().await {
                    if state == VerificationState::Verified {
                        break;
                    }
                }
            };

            select! {
                _ = verified => cancel.cancel(),
                _ = run_to_device_sync(client.clone(), cancel.clone()) => {}
            }
        });
    }

    pub(crate) async fn store_credentials(&self) -> Result<(), BackendError> {
        store_session(&self.login_id, &self.client, &self.credential_repository).await
    }

    /// Closed is the same thing as the session's tasks being cancelled.
    fn ensure_open(&self) -> Result<(), BackendError> {
        if self.tasks_token.is_cancelled() {
            return Err(technical("session closed"));
        }
        Ok(())
    }

    fn shutdown(&self) {
        self.tasks_token.cancel();
        drop(lock(&self.verification).take());
        drop(lock(&self.reset).take());
    }

    fn user_id(&self) -> Result<&matrix_sdk::ruma::UserId, BackendError> {
        self.client
            .user_id()
            .ok_or_else(|| technical("Client has no user id"))
    }

    async fn start_reset(&self) -> Result<IdentityReset, VerificationError> {
        let started = start_identity_reset(&self.client).await;

        let mut slot = lock(&self.reset);
        if !matches!(*slot, Some(PendingReset::Starting)) {
            return Err(technical("session closed").into());
        }
        match started {
            Ok(mut pending) => {
                let report = pending.observe();
                *slot = Some(pending);
                Ok(report.map_err(technical)?)
            }
            Err(e) => {
                *slot = None;
                Err(technical(e).into())
            }
        }
    }

    async fn reset_with_password(
        &self,
        handle: Arc<IdentityResetHandle>,
        uiaa_session: Option<String>,
        password: Password,
    ) -> Result<IdentityReset, VerificationError> {
        let auth = password_auth(self.user_id()?, uiaa_session, &password);
        let recovery_key = run_reset(&self.client, &handle, Some(auth))
            .await
            .map_err(technical)?;

        let mut slot = lock(&self.reset);
        if matches!(*slot, Some(PendingReset::PasswordRequired { .. })) {
            *slot = Some(PendingReset::Done {
                recovery_key: recovery_key.clone(),
            });
        }
        Ok(IdentityReset::Done { recovery_key })
    }
}

enum ResetStep {
    Report(Result<IdentityReset, VerificationError>),
    Start,
    Password {
        handle: Arc<IdentityResetHandle>,
        uiaa_session: Option<String>,
        password: Password,
    },
}

#[async_trait]
impl BackendSession for BackendSessionMatrix {
    async fn authenticate(&self, credential: &Credential) -> Result<(), BackendError> {
        self.ensure_open()?;

        match credential {
            Credential::OAuthCallback(query) => {
                self.client
                    .oauth()
                    .finish_login(UrlOrQuery::Query(query.clone()))
                    .await
                    .map_err(technical)?;
            }
            Credential::Password(password) => {
                self.client
                    .matrix_auth()
                    .login_username(&self.user_id, password.as_str())
                    .send()
                    .await
                    .map_err(technical)?;
            }
        }

        self.store_credentials().await?;
        self.spawn_token_watcher();
        self.sync_until_verified();

        Ok(())
    }

    async fn device_status(&self) -> Result<DeviceStatus, BackendError> {
        self.ensure_open()?;

        let encryption = self.client.encryption();
        encryption.wait_for_e2ee_initialization_tasks().await;

        encryption
            .request_user_identity(self.user_id()?)
            .await
            .map_err(technical)?;

        let own_device = encryption.get_own_device().await.map_err(technical)?;

        let Some(own_device) = own_device else {
            return Ok(DeviceStatus::Unverified);
        };

        Ok(match own_device.is_cross_signed_by_owner() {
            true => DeviceStatus::Verified,
            false => DeviceStatus::Unverified,
        })
    }

    async fn wait_until_verified(&self) -> Result<(), BackendError> {
        // Subscribed before the status check so a flip landing during the query is not lost.
        // The SDK only recomputes this state after a `/keys/query` that covers our own
        // device; the check forces one, and everything that verifies the device afterwards
        // ends in another.
        let mut states = self.client.encryption().verification_state();
        if self.device_status().await? == DeviceStatus::Verified {
            self.pre_ready_sync.cancel();
            return Ok(());
        }

        loop {
            select! {
                _ = self.tasks_token.cancelled() => {
                    return Err(technical("session closed"));
                }
                state = states.next() => match state {
                    Some(VerificationState::Verified) => {
                        self.pre_ready_sync.cancel();
                        return Ok(());
                    }
                    Some(_) => {}
                    None => return Err(technical("the verification state stream ended")),
                },
            }
        }
    }

    async fn verification_options(&self) -> Result<VerificationOptions, VerificationError> {
        self.ensure_open()?;

        let encryption = self.client.encryption();
        let user_id = self.user_id()?;

        encryption
            .request_user_identity(user_id)
            .await
            .map_err(technical)?;

        let recovery_available = encryption
            .secret_storage()
            .is_enabled()
            .await
            .map_err(technical)?;

        let devices = encryption
            .get_user_devices(user_id)
            .await
            .map_err(technical)?
            .devices()
            .filter(|device| {
                device.is_cross_signed_by_owner()
                    && device.curve25519_key().is_some()
                    && !device.is_dehydrated()
            })
            .map(|device| DeviceSummary {
                id: DeviceId::new(device.device_id().as_str()),
                display_name: device.display_name().map(str::to_owned),
            })
            .collect();

        Ok(VerificationOptions {
            recovery_available,
            devices,
        })
    }

    async fn recover(&self, key: &RecoveryKey) -> Result<DeviceStatus, VerificationError> {
        self.ensure_open()?;

        self.client
            .encryption()
            .recovery()
            .recover(key.as_str())
            .await
            .map_err(map_recovery_error)?;

        Ok(self.device_status().await?)
    }

    async fn start_device_verification(&self, device: &DeviceId) -> Result<(), VerificationError> {
        self.ensure_open()?;

        let Ok(device_id) = device.clone().map_into();

        let device = self
            .client
            .encryption()
            .get_device(self.user_id()?, &device_id)
            .await
            .map_err(technical)?
            .ok_or_else(|| technical(format!("unknown device {device_id}")))?;

        let previous = lock(&self.verification).take();
        if let Some(previous) = previous {
            previous.cancel().await;
        }

        let flow = VerificationFlowMatrix::start(device).await?;

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
            VerificationAction::Cancel => request.cancel().await.map_err(technical)?,
            VerificationAction::Confirm => sas_of(&request)
                .ok_or(VerificationError::NoVerificationInProgress)?
                .confirm()
                .await
                .map_err(technical)?,
            VerificationAction::Mismatch => sas_of(&request)
                .ok_or(VerificationError::NoVerificationInProgress)?
                .mismatch()
                .await
                .map_err(technical)?,
        }
        Ok(())
    }

    async fn reset_identity(
        &self,
        auth: Option<ResetAuth>,
    ) -> Result<IdentityReset, VerificationError> {
        self.ensure_open()?;

        let step = {
            let mut slot = lock(&self.reset);
            match (auth, slot.as_mut()) {
                (None, None) => {
                    *slot = Some(PendingReset::Starting);
                    ResetStep::Start
                }
                (Some(_), None) => {
                    ResetStep::Report(Err(VerificationError::NoVerificationInProgress))
                }
                (None, Some(pending)) => ResetStep::Report(observed(pending.observe())),
                (
                    Some(ResetAuth::Password(password)),
                    Some(PendingReset::PasswordRequired {
                        handle,
                        uiaa_session,
                    }),
                ) => ResetStep::Password {
                    handle: handle.clone(),
                    uiaa_session: uiaa_session.clone(),
                    password,
                },
                (
                    Some(ResetAuth::Approved),
                    Some(pending @ PendingReset::ApprovalRequired { .. }),
                ) => ResetStep::Report(observed(pending.approve(&self.client))),
                (Some(_), Some(pending)) => ResetStep::Report(observed(pending.observe())),
            }
        };

        match step {
            ResetStep::Report(report) => report,
            ResetStep::Start => self.start_reset().await,
            ResetStep::Password {
                handle,
                uiaa_session,
                password,
            } => {
                self.reset_with_password(handle, uiaa_session, password)
                    .await
            }
        }
    }

    async fn close(&self) {
        self.shutdown();
    }

    async fn discard(&self) {
        self.discard_store.store(true, Ordering::SeqCst);
        self.shutdown();
    }

    async fn hard_log_out(&self) -> Result<(), BackendError> {
        if self.client.session().is_none() {
            return Ok(());
        }
        match tokio::time::timeout(LOGOUT_WINDOW, self.client.logout()).await {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(technical(e)),
            Err(_) => Err(technical(format!(
                "the homeserver did not acknowledge the logout within {}s",
                LOGOUT_WINDOW.as_secs()
            ))),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for BackendSessionMatrix {
    fn drop(&mut self) {
        self.shutdown();
        if self.discard_store.load(Ordering::SeqCst) {
            if let Some(login_dir) = self.login_dir.take() {
                remove_login_dir_once_released(login_dir);
            }
        }
    }
}

/// The SDK's sqlite handle is only released once the last `Client` clone is gone, and on
/// Windows the directory cannot be removed before that, so the removal is retried for a
/// while from a task rather than attempted inline.
fn remove_login_dir_once_released(login_dir: PathBuf) {
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        let _ = std::fs::remove_dir_all(&login_dir);
        return;
    };
    runtime.spawn(async move {
        for _ in 0..40 {
            if remove_login_dir(&login_dir).await.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        log::warn!(
            "Could not remove the store of a discarded login at {}",
            login_dir.display()
        );
    });
}

fn observed(report: anyhow::Result<IdentityReset>) -> Result<IdentityReset, VerificationError> {
    Ok(report.map_err(technical)?)
}

fn map_recovery_error(error: RecoveryError) -> VerificationError {
    match error {
        RecoveryError::SecretStorage(SecretStorageError::SecretStorageKey(_)) => {
            VerificationError::RecoveryKeyRejected
        }
        RecoveryError::SecretStorage(SecretStorageError::ImportError { name, error }) => {
            match error {
                ImportError::Sdk(_) | ImportError::Json(_) => {
                    technical(format!("importing {name}: {error}")).into()
                }
                ImportError::Key(_)
                | ImportError::MismatchedPublicKeys
                | ImportError::Decryption(_) => VerificationError::RecoveryKeyRejected,
            }
        }
        other => technical(other).into(),
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Writes the client's current tokens to the store under the login.
async fn store_session(
    login_id: &LoginId,
    client: &Client,
    credential_repository: &Arc<dyn CredentialRepository>,
) -> Result<(), BackendError> {
    let session = client
        .session()
        .ok_or_else(|| technical("Client has no session to persist"))?;

    let blob = SessionRestoreData::try_from(session)
        .map_err(BackendError::Technical)?
        .to_blob()
        .map_err(BackendError::Technical)?;

    credential_repository.store(login_id, blob).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::auth::AuthKind;
    use crate::infrastructure::backend::auth_service::tests::build_test_client;
    use tachyon_core::domain::verification::Password;
    use matrix_sdk::ruma::api::client::sync::sync_events::v5::Response as SlidingSyncResponse;
    use matrix_sdk::ruma::{device_id, user_id};
    use matrix_sdk::test_utils::mocks::MatrixMockServer;
    use tokio::time::timeout;
    use matrix_sdk::ruma::events::secret::request::SecretName;
    use matrix_sdk_crypto::secret_storage::DecodeError;
    use serde_json::json;
    use tachyon_testkit::repositories::CredentialRepositoryInMem;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn session(client: Client) -> BackendSessionMatrix {
        BackendSessionMatrix::new(
            client,
            LoginId::new("l1"),
            Arc::new(CredentialRepositoryInMem::default()),
            user_id!("@aeon:shlasouf.local").to_owned(),
            None,
        )
    }

    #[tokio::test]
    async fn a_password_login_stores_matrix_credentials() {
        let server = MatrixMockServer::new().await;
        server.mock_login().ok().mount().await;
        let client = server.client_builder().unlogged().build().await;
        let repository = Arc::new(CredentialRepositoryInMem::default());
        let session = BackendSessionMatrix::new(
            client,
            LoginId::new("l1"),
            repository.clone(),
            user_id!("@aeon:shlasouf.local").to_owned(),
            None,
        );

        session
            .authenticate(&Credential::Password(Password::new("hunter2")))
            .await
            .unwrap();

        let blob = repository
            .credentials(&LoginId::new("l1"))
            .await
            .unwrap()
            .expect("the login should be persisted");
        let restored = SessionRestoreData::from_blob(&blob).unwrap();
        assert!(matches!(restored.auth_kind, AuthKind::Matrix));
        assert!(!restored.access_token.is_empty());
    }

    #[tokio::test]
    async fn logging_out_ends_the_device_on_the_homeserver() {
        let server = MatrixMockServer::new().await;
        server.mock_login().ok().mount().await;
        mock_logout(&server).await;
        let client = server.client_builder().unlogged().build().await;
        let session = BackendSessionMatrix::new(
            client,
            LoginId::new("l1"),
            Arc::new(CredentialRepositoryInMem::default()),
            user_id!("@aeon:shlasouf.local").to_owned(),
            None,
        );
        session
            .authenticate(&Credential::Password(Password::new("hunter2")))
            .await
            .unwrap();

        session.hard_log_out().await.unwrap();

        assert_eq!(requests_to(&server, "/logout").await, 1);
    }

    /// The SDK mock's logout endpoint insists on its own default access token, which the
    /// mocked login does not hand out.
    async fn mock_logout(server: &MatrixMockServer) {
        Mock::given(method("POST"))
            .and(path("/_matrix/client/v3/logout"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
            .mount(server.server())
            .await;
    }

    #[tokio::test]
    async fn closing_or_discarding_an_authenticated_session_keeps_its_device() {
        let server = MatrixMockServer::new().await;
        server.mock_login().ok().mount().await;
        mock_logout(&server).await;
        let client = server.client_builder().unlogged().build().await;
        let session = BackendSessionMatrix::new(
            client,
            LoginId::new("l1"),
            Arc::new(CredentialRepositoryInMem::default()),
            user_id!("@aeon:shlasouf.local").to_owned(),
            None,
        );
        session
            .authenticate(&Credential::Password(Password::new("hunter2")))
            .await
            .unwrap();

        session.close().await;
        session.discard().await;

        assert_eq!(requests_to(&server, "/logout").await, 0);
    }

    async fn logged_in_session() -> (BackendSessionMatrix, MockServer) {
        let (client, mock) = build_test_client().await;
        client
            .restore_session(SessionRestoreData {
                access_token: "access".to_string(),
                refresh_token: None,
                user_id: matrix_sdk::ruma::UserId::parse("@aeon:shlasouf.local")
                    .unwrap()
                    .to_owned(),
                device_id: matrix_sdk::ruma::OwnedDeviceId::from("DEVICEID"),
                auth_kind: AuthKind::Matrix,
            })
            .await
            .unwrap();
        (session(client), mock)
    }

    async fn requests_made(mock: &MockServer) -> usize {
        mock.received_requests().await.unwrap().len()
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
    async fn approving_or_answering_a_reset_that_was_never_started_is_refused() {
        let (session, mock) = logged_in_session().await;

        assert!(matches!(
            session.reset_identity(Some(ResetAuth::Approved)).await,
            Err(VerificationError::NoVerificationInProgress)
        ));
        assert!(matches!(
            session
                .reset_identity(Some(ResetAuth::Password(Password::new("pw"))))
                .await,
            Err(VerificationError::NoVerificationInProgress)
        ));
        assert_eq!(requests_made(&mock).await, 0);
    }

    #[tokio::test]
    async fn a_reset_in_flight_is_reported_and_never_restarted() {
        let (session, mock) = logged_in_session().await;
        *lock(&session.reset) = Some(PendingReset::Starting);

        assert!(matches!(
            session.reset_identity(None).await,
            Ok(IdentityReset::ApprovalPending)
        ));
        assert!(matches!(
            session.reset_identity(Some(ResetAuth::Approved)).await,
            Ok(IdentityReset::ApprovalPending)
        ));
        assert_eq!(requests_made(&mock).await, 0);
    }

    #[tokio::test]
    async fn a_finished_reset_keeps_reporting_its_recovery_key() {
        let (session, mock) = logged_in_session().await;
        *lock(&session.reset) = Some(PendingReset::Done {
            recovery_key: RecoveryKey::new("EsTc"),
        });

        for auth in [None, Some(ResetAuth::Approved)] {
            let Ok(IdentityReset::Done { recovery_key }) = session.reset_identity(auth).await
            else {
                panic!("expected Done");
            };
            assert_eq!(recovery_key, RecoveryKey::new("EsTc"));
        }
        assert_eq!(requests_made(&mock).await, 0);
    }

    #[tokio::test]
    async fn a_failed_reset_start_frees_the_slot() {
        let (session, mock) = logged_in_session().await;

        assert!(matches!(
            session.reset_identity(None).await,
            Err(VerificationError::Backend(_))
        ));
        assert!(requests_made(&mock).await > 0);
        assert!(lock(&session.reset).is_none());
    }

    #[tokio::test]
    async fn recovery_failing_on_the_wire_is_a_backend_error() {
        let (session, _mock) = logged_in_session().await;

        assert!(matches!(
            session.recover(&RecoveryKey::new("EsTc")).await,
            Err(VerificationError::Backend(_))
        ));
    }

    #[tokio::test]
    async fn options_on_an_empty_store_offer_nothing() {
        let (session, mock) = logged_in_session().await;
        Mock::given(path("/_matrix/client/versions"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({ "versions": ["v1.11"] })),
            )
            .with_priority(1)
            .mount(&mock)
            .await;

        let options = session.verification_options().await.unwrap();

        assert!(!options.recovery_available);
        assert!(options.devices.is_empty());
    }

    #[test]
    fn only_a_key_that_does_not_open_the_store_is_rejected() {
        let rejected = [
            RecoveryError::SecretStorage(SecretStorageError::SecretStorageKey(
                DecodeError::KeyLength(32, 3),
            )),
            RecoveryError::SecretStorage(SecretStorageError::ImportError {
                name: SecretName::CrossSigningMasterKey,
                error: ImportError::MismatchedPublicKeys,
            }),
        ];
        for error in rejected {
            assert!(matches!(
                map_recovery_error(error),
                VerificationError::RecoveryKeyRejected
            ));
        }

        let technical = [
            RecoveryError::BackupExistsOnServer,
            RecoveryError::SecretStorage(SecretStorageError::MissingKeyInfo { key_id: None }),
        ];
        for error in technical {
            assert!(matches!(
                map_recovery_error(error),
                VerificationError::Backend(BackendError::Technical(_))
            ));
        }
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
        let session = BackendSessionMatrix::new(
            client,
            login_id.clone(),
            repository.clone(),
            restore_data.user_id.clone(),
            None,
        );

        session.store_credentials().await.unwrap();

        let blob = repository.credentials(&login_id).await.unwrap().unwrap();
        let restored = SessionRestoreData::from_blob(&blob).unwrap();
        assert!(restored == restore_data);
    }

    /// A client with a real in-memory crypto store against a server that serves back whatever
    /// keys and signatures the client uploads. The device's keys are already on the server.
    async fn crypto_session() -> (Arc<BackendSessionMatrix>, Client, MatrixMockServer) {
        let (session, client, server) = fresh_crypto_session().await;
        server.mock_sync().ok_and_run(&client, |_| {}).await;
        (session, client, server)
    }

    /// Like `crypto_session`, but nothing has been uploaded yet: the shape right after an
    /// interactive login.
    async fn fresh_crypto_session() -> (Arc<BackendSessionMatrix>, Client, MatrixMockServer) {
        let server = MatrixMockServer::new().await;
        server.mock_crypto_endpoints_preset().await;
        server.mock_versions().with_simplified_sliding_sync().ok().mount().await;
        server
            .mock_sliding_sync()
            .ok(SlidingSyncResponse::new("pos".to_owned()))
            .mount()
            .await;
        let client = server
            .client_builder_for_crypto_end_to_end(
                user_id!("@aeon:shlasouf.local"),
                device_id!("DEVICEID"),
            )
            .no_server_versions()
            .build()
            .await;
        (Arc::new(session(client.clone())), client, server)
    }

    async fn requests_to(server: &MatrixMockServer, path_end: &str) -> usize {
        server
            .server()
            .received_requests()
            .await
            .unwrap()
            .iter()
            .filter(|request| request.url.path().ends_with(path_end))
            .count()
    }

    #[tokio::test]
    async fn a_fresh_device_uploads_its_keys_while_it_waits_to_be_verified() {
        let (session, _client, server) = fresh_crypto_session().await;
        assert_eq!(requests_to(&server, "/keys/upload").await, 0);

        session.sync_until_verified();

        let uploaded = timeout(Duration::from_secs(5), async {
            while requests_to(&server, "/keys/upload").await == 0 {
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await;
        assert!(uploaded.is_ok(), "the device keys never reached the homeserver");
    }

    async fn still_waiting(task: &tokio::task::JoinHandle<Result<(), BackendError>>) -> bool {
        tokio::time::sleep(Duration::from_millis(300)).await;
        !task.is_finished()
    }

    #[tokio::test]
    async fn wait_until_verified_returns_at_once_for_a_cross_signed_device() {
        let (session, client, _server) = crypto_session().await;
        client.encryption().bootstrap_cross_signing(None).await.unwrap();

        timeout(Duration::from_secs(5), session.wait_until_verified())
            .await
            .expect("a verified device should not keep the caller waiting")
            .unwrap();
    }

    #[tokio::test]
    async fn wait_until_verified_wakes_when_a_sync_brings_the_signature() {
        let (session, client, server) = crypto_session().await;
        let waiting = tokio::spawn({
            let session = session.clone();
            async move { session.wait_until_verified().await }
        });
        assert!(still_waiting(&waiting).await, "an unverified device must keep the caller waiting");

        client.encryption().bootstrap_cross_signing(None).await.unwrap();
        assert!(still_waiting(&waiting).await, "signing alone is not visible until keys are queried");

        let user_id = client.user_id().unwrap().to_owned();
        server
            .mock_sync()
            .ok_and_run(&client, |builder| {
                builder.add_change_device(&user_id);
            })
            .await;

        timeout(Duration::from_secs(5), waiting)
            .await
            .expect("the device-list change should wake the wait")
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn closing_the_session_ends_the_wait() {
        let (session, _client, _server) = crypto_session().await;
        let waiting = tokio::spawn({
            let session = session.clone();
            async move { session.wait_until_verified().await }
        });
        assert!(still_waiting(&waiting).await);

        session.close().await;

        let outcome = timeout(Duration::from_secs(5), waiting)
            .await
            .expect("closing should release the wait")
            .unwrap();
        assert!(matches!(outcome, Err(BackendError::Technical(_))), "{outcome:?}");
    }
}
