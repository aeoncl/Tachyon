//! Fakes for the ports the use cases depend on.
//!
//! Several of them yield at their await points. Without that, a fake resolves in one poll
//! and two concurrent callers never interleave, so a test could not tell whether the
//! lifecycle mutex is doing anything.

use crate::application::error::{BackendError, StoreError, VerificationError};
use crate::application::ports::{AccountRepository, AuthService, BackendSession};
use crate::domain::auth::{BridgeMetadata, InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::{DeviceId, LoginId, UserId};
use crate::domain::verification::{
    DeviceStatus, IdentityReset, RecoveryKey, ResetAuth, VerificationAction, VerificationFlowState,
    VerificationOptions,
};
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

/// A session whose device status follows a script: every call consumes the next entry and
/// the last one repeats, so a test can make the device flip to verified partway through.
pub(crate) struct FakeBackendSession {
    device_statuses: Mutex<Vec<DeviceStatus>>,
    device_status_fails: bool,
    verification_state: Mutex<Option<VerificationFlowState>>,
    device_status_calls: AtomicUsize,
    finish_calls: AtomicUsize,
    close_calls: AtomicUsize,
}

impl FakeBackendSession {
    pub(crate) fn new(device_statuses: impl IntoIterator<Item = DeviceStatus>) -> Arc<Self> {
        let device_statuses: Vec<_> = device_statuses.into_iter().collect();
        assert!(!device_statuses.is_empty(), "script at least one device status");
        Arc::new(Self {
            device_statuses: Mutex::new(device_statuses),
            device_status_fails: false,
            verification_state: Mutex::new(None),
            device_status_calls: AtomicUsize::new(0),
            finish_calls: AtomicUsize::new(0),
            close_calls: AtomicUsize::new(0),
        })
    }

    /// A session whose homeserver cannot be reached for the device status.
    pub(crate) fn with_unreachable_device_status() -> Arc<Self> {
        Arc::new(Self {
            device_statuses: Mutex::new(vec![DeviceStatus::Unverified]),
            device_status_fails: true,
            verification_state: Mutex::new(None),
            device_status_calls: AtomicUsize::new(0),
            finish_calls: AtomicUsize::new(0),
            close_calls: AtomicUsize::new(0),
        })
    }

    pub(crate) fn with_verification_state(
        self: Arc<Self>,
        state: VerificationFlowState,
    ) -> Arc<Self> {
        *self.verification_state.lock().unwrap() = Some(state);
        self
    }

    pub(crate) fn device_status_calls(&self) -> usize {
        self.device_status_calls.load(Ordering::SeqCst)
    }

    pub(crate) fn finish_calls(&self) -> usize {
        self.finish_calls.load(Ordering::SeqCst)
    }

    pub(crate) fn close_calls(&self) -> usize {
        self.close_calls.load(Ordering::SeqCst)
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
    async fn finish_interactive_login(&self, _callback_query: &str) -> Result<(), BackendError> {
        self.finish_calls.fetch_add(1, Ordering::SeqCst);
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
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Hands out one prepared session and counts how often it was asked to build one.
pub(crate) struct FakeAuthService {
    session: Arc<FakeBackendSession>,
    restore_calls: AtomicUsize,
    start_calls: AtomicUsize,
}

impl FakeAuthService {
    pub(crate) fn new(session: Arc<FakeBackendSession>) -> Arc<Self> {
        Arc::new(Self {
            session,
            restore_calls: AtomicUsize::new(0),
            start_calls: AtomicUsize::new(0),
        })
    }

    pub(crate) fn restore_calls(&self) -> usize {
        self.restore_calls.load(Ordering::SeqCst)
    }

    pub(crate) fn start_calls(&self) -> usize {
        self.start_calls.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl AuthService for FakeAuthService {
    async fn restore(&self, _login_id: &LoginId) -> Result<Arc<dyn BackendSession>, BackendError> {
        self.restore_calls.fetch_add(1, Ordering::SeqCst);
        tokio::task::yield_now().await;
        Ok(self.session.clone())
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
            self.session.clone(),
            InteractiveAuthStarted::OAuth {
                auth_url: "https://auth.example/authorize".to_string(),
                csrf_token: "csrf".to_string(),
            },
        ))
    }
}

#[derive(Default)]
pub(crate) struct FakeAccountRepository {
    logins_by_token: Mutex<HashMap<TachyonToken, LoginId>>,
}

impl FakeAccountRepository {
    pub(crate) fn with_login(token: &TachyonToken, login_id: &LoginId) -> Arc<Self> {
        let repository = Self::default();
        repository
            .logins_by_token
            .lock()
            .unwrap()
            .insert(token.clone(), login_id.clone());
        Arc::new(repository)
    }
}

#[async_trait]
impl AccountRepository for FakeAccountRepository {
    async fn login_id_by_token(
        &self,
        tachyon_token: &TachyonToken,
    ) -> Result<Option<LoginId>, StoreError> {
        tokio::task::yield_now().await;
        Ok(self
            .logins_by_token
            .lock()
            .unwrap()
            .get(tachyon_token)
            .cloned())
    }

    async fn save_login_for_token(
        &self,
        tachyon_token: TachyonToken,
        login_id: LoginId,
    ) -> Result<(), StoreError> {
        self.logins_by_token
            .lock()
            .unwrap()
            .insert(tachyon_token, login_id);
        Ok(())
    }
}
