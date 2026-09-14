use crate::application::error::AuthError;
use crate::application::logins::{Login, Logins, Step};
use crate::application::ports::{AccountRepository, AuthService, BackendSession};
use crate::application::web_urls::WebUrls;
use crate::domain::auth::{
    ending, BridgeLinkToken, BridgeMetadata, Credential, Effect, Ending, InteractiveAuthStarted,
};
use crate::domain::ids::{LoginId, UserId};
use crate::domain::verification::DeviceStatus;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub enum SignIn {
    Ready(Arc<dyn BackendSession>),
    Pending { step: Step, url: String },
}

#[derive(Debug)]
pub struct FinishedLogin {
    pub token: BridgeLinkToken,
    pub device_status: DeviceStatus,
    pub redirect_url: Option<String>,
}

pub struct AuthUseCase {
    account_repository: Arc<dyn AccountRepository>,
    auth_service: Arc<dyn AuthService>,
    logins: Arc<Logins>,
    /// Guards everything that creates, advances or drops a login, so two callers cannot
    /// each build a backend session for the same token.
    login_guard: Mutex<()>,
    web_urls: WebUrls,
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
            login_guard: Mutex::new(()),
            web_urls: web,
        }
    }

    pub async fn sign_in_or_restore_login(
        &self,
        token: &BridgeLinkToken,
        server_name: &str,
        user_id: UserId,
        bridge_metadata: &BridgeMetadata,
    ) -> Result<SignIn, AuthError> {
        let _guard = self.login_guard.lock().await;

        if self.logins.contains(token) {
            return Err(AuthError::AlreadySignedIn);
        }

        let Some(login_id) = self.account_repository.login_id_by_token(token).await? else {
            return self
                .start_interactive_flow(token, server_name, user_id, bridge_metadata)
                .await;
        };

        let session = match self.auth_service.restore(&login_id).await {
            Ok(session) => session,
            Err(e) if e.ends_the_login() => {
                self.remove_stored_login(&login_id).await?;
                return self
                    .start_interactive_flow(token, server_name, user_id, bridge_metadata)
                    .await;
            }
            Err(e) => return Err(e.into()),
        };
        let device_status = match session.device_status().await {
            Ok(device_status) => device_status,
            Err(e) => {
                session.close().await;
                return Err(e.into());
            }
        };
        self.logins.insert(
            token.clone(),
            Login::after_auth(session.clone(), login_id, device_status),
        );
        Ok(match device_status {
            DeviceStatus::Verified => SignIn::Ready(session),
            DeviceStatus::Unverified => SignIn::Pending {
                step: Step::VerifyDevice,
                url: self.web_urls.confirm_device(token),
            },
        })
    }


    /// Waits until a session is ready, or time out.
    pub async fn wait_until_session_ready(
        &self,
        token: &BridgeLinkToken,
    ) -> Result<Arc<dyn BackendSession>, AuthError> {
        loop {
            let (session, login_id, step, changed) = match self.logins.get(token) {
                None => return Err(AuthError::LoginNotFound),
                Some(Login::Ready { session, .. }) => return Ok(session),
                Some(Login::Pending {
                    session,
                    login_id,
                    step,
                    changed,
                }) => (session, login_id, step, changed),
            };

            match step {
                Step::Authenticate { .. } => changed.notified().await,
                Step::VerifyDevice => {
                    session.wait_until_verified().await?;
                    return self.promote_login_to_ready(token, session, login_id).await;
                }
            }
        }
    }

    async fn promote_login_to_ready(
        &self,
        token: &BridgeLinkToken,
        session: Arc<dyn BackendSession>,
        login_id: LoginId,
    ) -> Result<Arc<dyn BackendSession>, AuthError> {
        let _guard = self.login_guard.lock().await;
        let ready = Login::ready(session.clone(), login_id);
        if self.logins.replace_if(token, &session, ready) {
            Ok(session)
        } else {
            Err(AuthError::LoginNotFound)
        }
    }

    async fn settle_device_status_after_authenticated(
        &self,
        token: &BridgeLinkToken,
        session: &Arc<dyn BackendSession>,
        login_id: LoginId,
    ) -> Result<DeviceStatus, AuthError> {
        let _guard = self.login_guard.lock().await;
        if !self.logins.holds(token, session) {
            self.execute_effects(session, &login_id, ending(Ending::Orphaned)).await?;
            return Err(AuthError::LoginNotFound);
        }

        let device_status = match self.link_login_and_read_device_status(token, session, &login_id).await {
            Ok(device_status) => device_status,
            Err(e) => {
                self.logins.remove_if(token, session);
                self.execute_effects(session, &login_id, ending(Ending::Unsettled)).await?;
                return Err(e);
            }
        };

        let advanced = Login::after_auth(session.clone(), login_id, device_status);
        if self.logins.replace_if(token, session, advanced) {
            Ok(device_status)
        } else {
            Err(AuthError::LoginNotFound)
        }
    }

    async fn link_login_and_read_device_status(
        &self,
        token: &BridgeLinkToken,
        session: &Arc<dyn BackendSession>,
        login_id: &LoginId,
    ) -> Result<DeviceStatus, AuthError> {
        self.account_repository
            .save_login_for_token(token.clone(), login_id.clone())
            .await?;
        Ok(session.device_status().await?)
    }

    pub async fn abandon_login(&self, token: &BridgeLinkToken) -> Result<(), AuthError> {
        let _guard = self.login_guard.lock().await;
        match self.logins.remove(token) {
            Some(login) => self.drop_login(login).await,
            None => Ok(()),
        }
    }

    /// Remove everything for a login and explictely logs-out on the server (removing the Device)
    pub async fn hard_logout(&self, token: &BridgeLinkToken) -> Result<(), AuthError> {
        let _guard = self.login_guard.lock().await;
        if let Some(login) = self.logins.remove(token) {
            return self.end_login(login, Ending::Deleted).await;
        }
        let Some(login_id) = self.account_repository.login_id_by_token(token).await? else {
            return Ok(());
        };

        // Logging out needs a client with the login's tokens; one that cannot be rebuilt
        // has no device left to end anyway.
        match self.auth_service.restore(&login_id).await {
            Ok(session) => {
                self.execute_effects(&session, &login_id, ending(Ending::Deleted)).await?;
                self.auth_service.remove_login(&login_id).await?;
                Ok(())
            }
            Err(_) => self.remove_stored_login(&login_id).await,
        }
    }

    pub async fn clear_unlinked_logins(&self) -> Result<(), AuthError> {
        let _guard = self.login_guard.lock().await;

        let mut keep = Vec::new();
        for stored in self.account_repository.logins().await? {
            if stored.linked {
                keep.push(stored.login_id);
            } else {
                self.remove_stored_login(&stored.login_id).await?;
            }
        }
        self.auth_service.clear_logins_except(&keep).await?;
        Ok(())
    }

    async fn drop_login(&self, login: Login) -> Result<(), AuthError> {
        let why = Ending::Dropped {
            authenticated: login.authenticated(),
        };
        self.end_login(login, why).await
    }

    async fn end_login(&self, login: Login, why: Ending) -> Result<(), AuthError> {
        if let Login::Pending { changed, .. } = &login {
            changed.notify_one();
        }
        self.execute_effects(login.session(), login.login_id(), ending(why))
            .await
    }

    async fn execute_effects(
        &self,
        session: &Arc<dyn BackendSession>,
        login_id: &LoginId,
        effects: Vec<Effect>,
    ) -> Result<(), AuthError> {
        for effect in effects {
            match effect {
                Effect::Close => session.close().await,
                Effect::Discard => session.discard().await,
                Effect::LogOut => {
                    if let Err(e) = session.hard_log_out().await {
                        log::warn!("Could not log the device out, ending it anyway: {e:?}");
                    }
                }
                Effect::DeleteRow => self.account_repository.delete_login(login_id).await?,
            }
        }
        Ok(())
    }

    async fn remove_stored_login(&self, login_id: &LoginId) -> Result<(), AuthError> {
        self.auth_service.remove_login(login_id).await?;
        self.account_repository.delete_login(login_id).await?;
        Ok(())
    }

    pub fn ready_session(&self, token: &BridgeLinkToken) -> Option<Arc<dyn BackendSession>> {
        self.logins.ready_session(token)
    }

    pub fn has_login(&self, token: &BridgeLinkToken) -> bool {
        self.logins.contains(token)
    }


}

impl AuthUseCase {

    async fn start_interactive_flow(
        &self,
        token: &BridgeLinkToken,
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
                &self.web_urls.oauth_callback(),
                bridge_metadata,
            )
            .await?;

        let flow_id = match &prompt {
            InteractiveAuthStarted::OAuth { csrf_token, .. } => csrf_token.clone(),
            InteractiveAuthStarted::PasswordRequired => Uuid::new_v4().simple().to_string(),
        };
        let url = self.web_urls.login_start(&flow_id);
        let step = Step::Authenticate { flow_id, prompt };
        self.logins.insert(
            token.clone(),
            Login::pending(session, login_id, step.clone()),
        );
        Ok(SignIn::Pending { step, url })
    }


    pub async fn abandon_interactive_flow(&self, flow_id: &str) -> Result<(), AuthError> {
        let _guard = self.login_guard.lock().await;
        let Some(token) = self.logins.token_for_flow(flow_id) else {
            return Ok(());
        };
        match self.logins.remove(&token) {
            Some(login) => self.drop_login(login).await,
            None => Ok(()),
        }
    }

    pub async fn finish_interactive_flow(
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

        let finished = self.settle_device_status_after_authenticated(&token, &session, login_id).await;
        changed.notify_one();
        finished.map(|device_status| FinishedLogin {
            redirect_url: match device_status {
                DeviceStatus::Verified => None,
                DeviceStatus::Unverified => Some(self.web_urls.confirm_device(&token)),
            },
            token,
            device_status,
        })
    }

    pub fn prompt_for_interactive_flow(&self, flow_id: &str) -> Option<InteractiveAuthStarted> {
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
