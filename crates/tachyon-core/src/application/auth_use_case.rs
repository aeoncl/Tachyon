use crate::application::error::{AuthError, BackendError};
use crate::application::logins::{Login, Logins, Step};
use crate::application::ports::{AccountRepository, AuthService, BackendSession};
use crate::domain::auth::{BridgeMetadata, Credential, InteractiveAuthStarted, TachyonToken};
use crate::domain::ids::{LoginId, UserId};
use crate::domain::verification::DeviceStatus;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub enum SignIn {
    Ready(Arc<dyn BackendSession>),
    /// The user has to do something in a browser first, at `url`.
    /// `AuthUseCase::wait_for_session` resolves once they have.
    Pending { step: Step, url: String },
}

/// The bridge's web pages as the user's browser reaches them. Every URL a backend or a user
/// is sent to is built from this one base, so the pages and the links agree by construction.
pub struct WebUrls {
    base: String,
}

impl WebUrls {
    /// `base` is the pages' root, for instance `http://127.0.0.1:11866/tachyon`.
    pub fn new(base: impl Into<String>) -> Self {
        let base: String = base.into();
        Self {
            base: base.trim_end_matches('/').to_owned(),
        }
    }

    /// Where the authorization server sends the browser back after OAuth.
    fn oauth_callback(&self) -> String {
        format!("{}/login/callback", self.base)
    }

    fn login_start(&self, flow_id: &str) -> String {
        format!(
            "{}/login/start?flow={}",
            self.base,
            urlencoding::encode(flow_id)
        )
    }

    fn confirm_device(&self, token: &TachyonToken) -> String {
        format!("{}/confirm_device?t={}", self.base, token.as_str())
    }
}

pub struct FinishedLogin {
    pub token: TachyonToken,
    pub device_status: DeviceStatus,
}

pub struct AuthUseCase {
    account_repository: Arc<dyn AccountRepository>,
    auth_service: Arc<dyn AuthService>,
    logins: Arc<Logins>,
    /// Serializes everything that creates, advances or drops a login, so two callers cannot
    /// each build a backend session for the same token. A token names one bridge's login;
    /// two bridges signing the same account in hold two tokens and two logins.
    lifecycle: Mutex<()>,
    web: WebUrls,
}

impl AuthUseCase {
    pub fn new(
        account_repository: Arc<dyn AccountRepository>,
        logins: Arc<Logins>,
        auth_service: Arc<dyn AuthService>,
        web: WebUrls,
    ) -> AuthUseCase {
        AuthUseCase {
            account_repository,
            auth_service,
            logins,
            lifecycle: Mutex::new(()),
            web,
        }
    }

    /// Opens the account's session, rebuilding it from the store when one is persisted and
    /// starting an interactive login when none is. A login already pending for the token is
    /// dropped and started over.
    pub async fn sign_in(
        &self,
        token: &TachyonToken,
        server_name: &str,
        user_id: UserId,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<SignIn, AuthError> {
        let _guard = self.lifecycle.lock().await;

        match self.logins.get(token) {
            Some(Login::Ready { session, .. }) => return Ok(SignIn::Ready(session)),
            Some(pending @ Login::Pending { .. }) => {
                self.logins.remove(token);
                self.drop_login(pending).await?;
            }
            None => {}
        }

        let Some(login_id) = self.account_repository.login_id_by_token(token).await? else {
            return self
                .start_interactive_login(token, server_name, user_id, bridge_metadata)
                .await;
        };

        let session = match self.auth_service.restore(&login_id).await {
            Ok(session) => session,
            // The backend no longer honours this login, so nothing about it is worth
            // keeping and the user has to sign in afresh. A backend that merely cannot be
            // reached right now is a different matter and must not cost them the login.
            Err(
                BackendError::LoggedOut
                | BackendError::SoftLoggedOut
                | BackendError::CannotRestoreLogin(_),
            ) => {
                self.forget_stored(&login_id).await?;
                return self
                    .start_interactive_login(token, server_name, user_id, bridge_metadata)
                    .await;
            }
            Err(e) => return Err(e.into()),
        };
        match session.device_status().await {
            Ok(DeviceStatus::Verified) => {
                self.logins.insert(
                    token.clone(),
                    Login::Ready {
                        session: session.clone(),
                        login_id,
                    },
                );
                Ok(SignIn::Ready(session))
            }
            Ok(DeviceStatus::Unverified) => {
                self.logins.insert(
                    token.clone(),
                    Login::pending(session, login_id, Step::VerifyDevice),
                );
                Ok(SignIn::Pending {
                    step: Step::VerifyDevice,
                    url: self.web.confirm_device(token),
                })
            }
            Err(e) => {
                session.close().await;
                Err(e.into())
            }
        }
    }

    async fn start_interactive_login(
        &self,
        token: &TachyonToken,
        server_name: &str,
        user_id: UserId,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<SignIn, AuthError> {
        let login_id = LoginId::new(Uuid::new_v4().to_string());
        let (session, prompt) = self
            .auth_service
            .start_interactive_login(
                &login_id,
                server_name,
                Some(user_id),
                &self.web.oauth_callback(),
                bridge_metadata,
            )
            .await?;

        let flow_id = match &prompt {
            InteractiveAuthStarted::OAuth { csrf_token, .. } => csrf_token.clone(),
            InteractiveAuthStarted::PasswordRequired => Uuid::new_v4().simple().to_string(),
        };
        let url = self.web.login_start(&flow_id);
        let step = Step::Authenticate { flow_id, prompt };
        self.logins.insert(
            token.clone(),
            Login::pending(session, login_id, step.clone()),
        );
        Ok(SignIn::Pending { step, url })
    }

    /// Blocks until the pending login for `token` is usable, then hands out its session.
    /// Fails when the login is abandoned or replaced while waiting.
    pub async fn wait_for_session(
        &self,
        token: &TachyonToken,
    ) -> Result<Arc<dyn BackendSession>, AuthError> {
        loop {
            let (session, step, changed) = match self.logins.get(token) {
                None => return Err(AuthError::LoginNotFound),
                Some(Login::Ready { session, .. }) => return Ok(session),
                Some(Login::Pending {
                    session,
                    step,
                    changed,
                    ..
                }) => (session, step, changed),
            };

            match step {
                Step::Authenticate { .. } => changed.notified().await,
                Step::VerifyDevice => {
                    session.wait_until_verified().await?;
                    return self.promote(token, &session).await;
                }
            }
        }
    }

    /// Marks the login ready, unless it was abandoned or replaced while the device was
    /// being verified.
    async fn promote(
        &self,
        token: &TachyonToken,
        session: &Arc<dyn BackendSession>,
    ) -> Result<Arc<dyn BackendSession>, AuthError> {
        let _guard = self.lifecycle.lock().await;
        match self.logins.get(token) {
            Some(Login::Pending {
                session: current,
                login_id,
                ..
            }) if Arc::ptr_eq(&current, session) => {
                self.logins.insert(
                    token.clone(),
                    Login::Ready {
                        session: session.clone(),
                        login_id,
                    },
                );
                Ok(session.clone())
            }
            Some(Login::Ready {
                session: current, ..
            }) if Arc::ptr_eq(&current, session) => Ok(current),
            _ => Err(AuthError::LoginNotFound),
        }
    }

    /// Completes the login behind `flow_id` with what the user supplied and binds the token
    /// to it so later sign-ins restore it. A credential the backend rejects leaves the login
    /// pending so the user can try again; a failure after that abandons it, which releases
    /// whoever is waiting on it.
    pub async fn finish_login(
        &self,
        flow_id: &str,
        credential: Credential,
    ) -> Result<FinishedLogin, AuthError> {
        let Some(token) = self.logins.token_for_flow(flow_id) else {
            return Err(AuthError::LoginNotFound);
        };
        let Some(Login::Pending {
            session,
            login_id,
            step: Step::Authenticate { .. },
            changed,
        }) = self.logins.get(&token)
        else {
            return Err(AuthError::LoginNotFound);
        };

        session.authenticate(&credential).await?;

        let finished = self.settle_authenticated(&token, &session, login_id).await;
        if finished.is_err() {
            if let Err(e) = self.abandon(&token).await {
                log::warn!("Could not abandon a login that failed to settle: {e:?}");
            }
        }
        changed.notify_one();
        finished.map(|device_status| FinishedLogin {
            token,
            device_status,
        })
    }

    async fn settle_authenticated(
        &self,
        token: &TachyonToken,
        session: &Arc<dyn BackendSession>,
        login_id: LoginId,
    ) -> Result<DeviceStatus, AuthError> {
        let _guard = self.lifecycle.lock().await;
        self.account_repository
            .save_login_for_token(token.clone(), login_id.clone())
            .await?;
        let device_status = session.device_status().await?;

        let advanced = match device_status {
            DeviceStatus::Verified => Login::Ready {
                session: session.clone(),
                login_id,
            },
            DeviceStatus::Unverified => {
                Login::pending(session.clone(), login_id, Step::VerifyDevice)
            }
        };
        match self.logins.get(token) {
            Some(Login::Pending {
                session: current, ..
            }) if Arc::ptr_eq(&current, session) => {
                self.logins.insert(token.clone(), advanced);
                Ok(device_status)
            }
            _ => Err(AuthError::LoginNotFound),
        }
    }

    /// Closes the backend session and drops the live login, whatever step it is at. What a
    /// bridge does when its client disconnects: a login that can be restored keeps its
    /// store row, its device and its on-disk state. Nothing to do when there is none.
    pub async fn abandon(&self, token: &TachyonToken) -> Result<(), AuthError> {
        let _guard = self.lifecycle.lock().await;
        match self.logins.remove(token) {
            Some(login) => self.drop_login(login).await,
            None => Ok(()),
        }
    }

    /// The authorization server refused the login behind `flow_id`, so nobody can finish it.
    pub async fn abandon_flow(&self, flow_id: &str) -> Result<(), AuthError> {
        match self.logins.token_for_flow(flow_id) {
            Some(token) => self.abandon(&token).await,
            None => Ok(()),
        }
    }

    /// Ends the account's login for good, live or not: the device is logged out of the
    /// backend, and everything kept for the login is removed. The next sign-in starts from
    /// scratch. This is the one place a device is ever logged out.
    ///
    /// This is where the Messenger client's "delete credentials" action should land, through
    /// an unauthenticated tachyon-web endpoint that derives the token from the sign-in
    /// address. Which URL the client calls for it is not known yet; finding out means
    /// reverse engineering the Passport DLL, so no endpoint exists so far.
    pub async fn forget(&self, token: &TachyonToken) -> Result<(), AuthError> {
        let _guard = self.lifecycle.lock().await;

        let stored = self.account_repository.login_id_by_token(token).await?;
        let login_id = match self.logins.remove(token) {
            Some(login) => {
                if let Login::Pending { changed, .. } = &login {
                    changed.notify_one();
                }
                Self::log_out_and_discard(login.session()).await;
                Some(login.login_id().clone())
            }
            None => {
                if let Some(login_id) = &stored {
                    // Logging out needs a client with the login's tokens; one that cannot
                    // be rebuilt has no device left to end anyway.
                    if let Ok(session) = self.auth_service.restore(login_id).await {
                        Self::log_out_and_discard(&session).await;
                    }
                    self.auth_service.forget(login_id).await?;
                }
                stored
            }
        };

        if let Some(login_id) = login_id {
            self.account_repository.delete_login(&login_id).await?;
        }
        Ok(())
    }

    async fn log_out_and_discard(session: &Arc<dyn BackendSession>) {
        if let Err(e) = session.log_out().await {
            log::warn!("Could not log the forgotten device out, discarding it anyway: {e:?}");
        }
        session.discard().await;
    }

    /// Clears out what interrupted sign-ins and crashes leave behind: store rows no token
    /// points at, and backend state no store row points at. For startup.
    pub async fn sweep(&self) -> Result<(), AuthError> {
        let _guard = self.lifecycle.lock().await;

        let mut keep = Vec::new();
        for stored in self.account_repository.logins().await? {
            if stored.bound {
                keep.push(stored.login_id);
            } else {
                self.forget_stored(&stored.login_id).await?;
            }
        }
        self.auth_service.sweep(&keep).await?;
        Ok(())
    }

    /// A login that never authenticated has nothing worth keeping; the others may be
    /// restored later.
    async fn drop_login(&self, login: Login) -> Result<(), AuthError> {
        match login {
            Login::Pending {
                changed,
                step: Step::Authenticate { .. },
                session,
                login_id,
            } => {
                changed.notify_one();
                session.discard().await;
                self.account_repository.delete_login(&login_id).await?;
            }
            Login::Pending {
                changed, session, ..
            } => {
                changed.notify_one();
                session.close().await;
            }
            Login::Ready { session, .. } => session.close().await,
        }
        Ok(())
    }

    async fn forget_stored(&self, login_id: &LoginId) -> Result<(), AuthError> {
        self.auth_service.forget(login_id).await?;
        self.account_repository.delete_login(login_id).await?;
        Ok(())
    }

    /// The account's session once it is ready. What bridges use after sign-in.
    pub fn session(&self, token: &TachyonToken) -> Option<Arc<dyn BackendSession>> {
        self.logins.ready_session(token)
    }

    /// Whether the token names a login at any step, ready or not.
    pub fn has_login(&self, token: &TachyonToken) -> bool {
        self.logins.contains(token)
    }

    /// What the browser has to be pointed at to complete the login behind `flow_id`.
    pub fn prompt(&self, flow_id: &str) -> Option<InteractiveAuthStarted> {
        let token = self.logins.token_for_flow(flow_id)?;
        match self.logins.get(&token)? {
            Login::Pending {
                step: Step::Authenticate { prompt, .. },
                ..
            } => Some(prompt),
            _ => None,
        }
    }
}
