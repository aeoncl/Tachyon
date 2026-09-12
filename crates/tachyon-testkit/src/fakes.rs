//! Fakes for the backend ports. They yield at their await points: without that a fake
//! resolves in one poll and two concurrent callers never interleave, so a test could not
//! tell whether a lifecycle lock is doing anything.

use async_trait::async_trait;
use std::any::Any;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tachyon_core::application::error::{BackendError, VerificationError};
use tachyon_core::application::ports::{AuthService, BackendSession};
use tachyon_core::domain::auth::{BridgeMetadata, Credential, InteractiveAuthStarted};
use tachyon_core::domain::ids::{DeviceId, LoginId, UserId};
use tachyon_core::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use tokio::sync::Notify;

/// A session whose device status follows a script: every call consumes the next entry and
/// the last one repeats, so a test can make the device flip to verified partway through.
/// `wait_until_verified` parks until the test calls `verify`, or fails once `close` runs.
pub struct FakeBackendSession {
    device_statuses: Mutex<Vec<DeviceStatus>>,
    device_status_fails: bool,
    rejects_credentials: bool,
    verification_state: Mutex<Option<VerificationFlowState>>,
    device_status_calls: AtomicUsize,
    finish_calls: AtomicUsize,
    close_calls: AtomicUsize,
    discard_calls: AtomicUsize,
    log_out_calls: AtomicUsize,
    verified: Notify,
    closed: Notify,
}

impl FakeBackendSession {
    pub fn new(device_statuses: impl IntoIterator<Item = DeviceStatus>) -> Arc<Self> {
        let device_statuses: Vec<_> = device_statuses.into_iter().collect();
        assert!(!device_statuses.is_empty(), "script at least one device status");
        Arc::new(Self::build(device_statuses, false, false))
    }

    /// A session whose homeserver cannot be reached for the device status.
    pub fn with_unreachable_device_status() -> Arc<Self> {
        Arc::new(Self::build(vec![DeviceStatus::Unverified], true, false))
    }

    /// A session whose homeserver refuses every credential.
    pub fn rejecting_credentials() -> Arc<Self> {
        Arc::new(Self::build(vec![DeviceStatus::Unverified], false, true))
    }

    fn build(
        device_statuses: Vec<DeviceStatus>,
        device_status_fails: bool,
        rejects_credentials: bool,
    ) -> Self {
        Self {
            device_statuses: Mutex::new(device_statuses),
            device_status_fails,
            rejects_credentials,
            verification_state: Mutex::new(None),
            device_status_calls: AtomicUsize::new(0),
            finish_calls: AtomicUsize::new(0),
            close_calls: AtomicUsize::new(0),
            discard_calls: AtomicUsize::new(0),
            log_out_calls: AtomicUsize::new(0),
            verified: Notify::new(),
            closed: Notify::new(),
        }
    }

    pub fn with_verification_state(self: Arc<Self>, state: VerificationFlowState) -> Arc<Self> {
        *self.verification_state.lock().unwrap() = Some(state);
        self
    }

    pub fn device_status_calls(&self) -> usize {
        self.device_status_calls.load(Ordering::SeqCst)
    }

    pub fn authenticate_calls(&self) -> usize {
        self.finish_calls.load(Ordering::SeqCst)
    }

    pub fn close_calls(&self) -> usize {
        self.close_calls.load(Ordering::SeqCst)
    }

    pub fn discard_calls(&self) -> usize {
        self.discard_calls.load(Ordering::SeqCst)
    }

    pub fn log_out_calls(&self) -> usize {
        self.log_out_calls.load(Ordering::SeqCst)
    }

    /// The device just became trusted, however that happened.
    pub fn verify(&self) {
        self.verified.notify_one();
    }

    fn next_device_status(&self) -> DeviceStatus {
        let mut scripted = self.device_statuses.lock().unwrap();
        if scripted.len() > 1 {
            scripted.remove(0)
        } else {
            scripted[0]
        }
    }
}

#[async_trait]
impl BackendSession for FakeBackendSession {
    async fn authenticate(&self, _credential: &Credential) -> Result<(), BackendError> {
        self.finish_calls.fetch_add(1, Ordering::SeqCst);
        if self.rejects_credentials {
            return Err(BackendError::Technical(anyhow::anyhow!(
                "credentials rejected"
            )));
        }
        Ok(())
    }

    async fn device_status(&self) -> Result<DeviceStatus, BackendError> {
        self.device_status_calls.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await;
        if self.device_status_fails {
            return Err(BackendError::Technical(anyhow::anyhow!(
                "device status unavailable"
            )));
        }
        Ok(self.next_device_status())
    }

    async fn wait_until_verified(&self) -> Result<(), BackendError> {
        tokio::select! {
            _ = self.verified.notified() => Ok(()),
            _ = self.closed.notified() => Err(BackendError::Technical(anyhow::anyhow!(
                "session closed"
            ))),
        }
    }

    async fn verification_options(&self) -> Result<VerificationOptions, VerificationError> {
        Ok(VerificationOptions {
            recovery_available: true,
            devices: Vec::new(),
        })
    }

    async fn recover(&self, _key: &RecoveryKey) -> Result<DeviceStatus, VerificationError> {
        Ok(self.next_device_status())
    }

    async fn start_device_verification(
        &self,
        _device: &DeviceId,
    ) -> Result<(), VerificationError> {
        Ok(())
    }

    fn verification_state(&self) -> Option<VerificationFlowState> {
        self.verification_state.lock().unwrap().clone()
    }

    async fn verification_action(
        &self,
        _action: VerificationAction,
    ) -> Result<(), VerificationError> {
        Ok(())
    }

    async fn reset_identity(
        &self,
        _auth: Option<ResetAuth>,
    ) -> Result<IdentityReset, VerificationError> {
        Ok(IdentityReset::ApprovalPending)
    }

    async fn close(&self) {
        self.close_calls.fetch_add(1, Ordering::SeqCst);
        self.closed.notify_one();
    }

    async fn discard(&self) {
        self.discard_calls.fetch_add(1, Ordering::SeqCst);
        self.closed.notify_one();
    }

    async fn log_out(&self) -> Result<(), BackendError> {
        self.log_out_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

enum Sessions {
    /// Every build hands out the same prepared session.
    Shared(Arc<FakeBackendSession>),
    /// Every build mints a fresh session with this device status script.
    Fresh(Vec<DeviceStatus>),
}

/// Builds sessions on demand and remembers every one it handed out, in order.
pub struct FakeAuthService {
    sessions: Sessions,
    handed_out: Mutex<Vec<Arc<FakeBackendSession>>>,
    restore_calls: AtomicUsize,
    start_calls: AtomicUsize,
    logged_out: AtomicBool,
    forgotten: Mutex<Vec<LoginId>>,
    swept_keeping: Mutex<Option<Vec<LoginId>>>,
}

impl FakeAuthService {
    pub fn handing_out(session: Arc<FakeBackendSession>) -> Arc<Self> {
        Arc::new(Self::build(Sessions::Shared(session)))
    }

    pub fn minting(device_statuses: impl IntoIterator<Item = DeviceStatus>) -> Arc<Self> {
        Arc::new(Self::build(Sessions::Fresh(
            device_statuses.into_iter().collect(),
        )))
    }

    fn build(sessions: Sessions) -> Self {
        Self {
            sessions,
            handed_out: Mutex::new(Vec::new()),
            restore_calls: AtomicUsize::new(0),
            start_calls: AtomicUsize::new(0),
            logged_out: AtomicBool::new(false),
            forgotten: Mutex::new(Vec::new()),
            swept_keeping: Mutex::new(None),
        }
    }

    /// From now on the backend reports every stored login as logged out.
    pub fn fail_restores_as_logged_out(&self) {
        self.logged_out.store(true, Ordering::SeqCst);
    }

    /// The logins `forget` was asked to drop, in order.
    pub fn forgotten(&self) -> Vec<LoginId> {
        self.forgotten.lock().unwrap().clone()
    }

    /// What the last `sweep` was told to keep, if one ran.
    pub fn swept_keeping(&self) -> Option<Vec<LoginId>> {
        self.swept_keeping.lock().unwrap().clone()
    }

    pub fn restore_calls(&self) -> usize {
        self.restore_calls.load(Ordering::SeqCst)
    }

    pub fn start_calls(&self) -> usize {
        self.start_calls.load(Ordering::SeqCst)
    }

    /// The `index`th session this service built.
    pub fn session(&self, index: usize) -> Arc<FakeBackendSession> {
        self.handed_out.lock().unwrap()[index].clone()
    }

    fn next_session(&self) -> Arc<FakeBackendSession> {
        let session = match &self.sessions {
            Sessions::Shared(session) => session.clone(),
            Sessions::Fresh(statuses) => FakeBackendSession::new(statuses.iter().copied()),
        };
        self.handed_out.lock().unwrap().push(session.clone());
        session
    }
}

#[async_trait]
impl AuthService for FakeAuthService {
    async fn restore(&self, _login_id: &LoginId) -> Result<Arc<dyn BackendSession>, BackendError> {
        self.restore_calls.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await;
        if self.logged_out.load(Ordering::SeqCst) {
            return Err(BackendError::LoggedOut);
        }
        Ok(self.next_session())
    }

    async fn forget(&self, login_id: &LoginId) -> Result<(), BackendError> {
        self.forgotten.lock().unwrap().push(login_id.clone());
        Ok(())
    }

    async fn sweep(&self, keep: &[LoginId]) -> Result<(), BackendError> {
        *self.swept_keeping.lock().unwrap() = Some(keep.to_vec());
        Ok(())
    }

    async fn start_interactive_login(
        &self,
        _login_id: &LoginId,
        _server_name: &str,
        _user_id: Option<UserId>,
        _redirect_url: &str,
        _bridge_metadata: &BridgeMetadata,
    ) -> Result<(Arc<dyn BackendSession>, InteractiveAuthStarted), BackendError> {
        self.start_calls.fetch_add(1, Ordering::SeqCst);
        Ok((
            self.next_session(),
            InteractiveAuthStarted::OAuth {
                auth_url: "https://auth.example/authorize".to_string(),
                csrf_token: "csrf".to_string(),
            },
        ))
    }
}
