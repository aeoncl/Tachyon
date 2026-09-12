//! The sign-in lifecycle over the real `Logins`, with the `tachyon_testkit` fakes standing in
//! for the backend and the store.

use tachyon_core::application::auth_use_case::{AuthUseCase, FinishedLogin, SignIn, WebUrls};
use tachyon_core::application::error::AuthError;
use tachyon_core::application::logins::{Logins, Step};
use tachyon_core::application::ports::{AccountRepository, BackendSession};
use tachyon_core::domain::auth::{BridgeMetadata, Credential, InteractiveAuthStarted, TachyonToken};
use tachyon_core::domain::verification::Password;
use tachyon_core::domain::ids::{LoginId, UserId};
use tachyon_core::domain::verification::DeviceStatus;
use std::sync::Arc;
use std::time::Duration;
use tachyon_testkit::fakes::{FakeAuthService, FakeBackendSession};
use tachyon_testkit::repositories::AccountRepositoryInMem;
use tokio::task::JoinHandle;
use tokio::time::timeout;

type Waiting = JoinHandle<Result<Arc<dyn BackendSession>, AuthError>>;

struct Fixture {
    use_case: Arc<AuthUseCase>,
    account_repository: Arc<AccountRepositoryInMem>,
    auth_service: Arc<FakeAuthService>,
    session: Arc<FakeBackendSession>,
    token: TachyonToken,
}

impl Fixture {
    /// An instance that has already authenticated the account once.
    fn with_stored_login(statuses: impl IntoIterator<Item = DeviceStatus>) -> Self {
        Self::new(FakeBackendSession::new(statuses), true)
    }

    /// An instance that has never seen the account.
    fn without_stored_login(statuses: impl IntoIterator<Item = DeviceStatus>) -> Self {
        Self::new(FakeBackendSession::new(statuses), false)
    }

    fn new(session: Arc<FakeBackendSession>, stored: bool) -> Self {
        let account_repository = if stored {
            AccountRepositoryInMem::with_login(&TachyonToken::new("tachyon-token"), &LoginId::new("login-1"))
        } else {
            Arc::new(AccountRepositoryInMem::default())
        };
        Self::over(session, account_repository)
    }

    fn over(session: Arc<FakeBackendSession>, account_repository: Arc<AccountRepositoryInMem>) -> Self {
        let token = TachyonToken::new("tachyon-token");
        let auth_service = FakeAuthService::handing_out(session.clone());
        let use_case = Arc::new(AuthUseCase::new(
            account_repository.clone(),
            Arc::new(Logins::default()),
            auth_service.clone(),
            WebUrls::new("https://bridge.example"),
        ));

        Self {
            use_case,
            account_repository,
            auth_service,
            session,
            token,
        }
    }

    async fn sign_in(&self) -> Result<SignIn, AuthError> {
        self.use_case
            .sign_in(
                &self.token,
                "example.org",
                UserId::new("@someone:example.org"),
                &bridge_metadata(),
            )
            .await
    }

    /// The pending step, or a panic with what came back instead.
    async fn sign_in_pending(&self) -> Step {
        self.sign_in_pending_with_url().await.0
    }

    async fn sign_in_pending_with_url(&self) -> (Step, String) {
        match self.sign_in().await {
            Ok(SignIn::Pending { step, url }) => (step, url),
            Ok(SignIn::Ready(_)) => panic!("the sign-in was ready"),
            Err(e) => panic!("the sign-in failed: {e:?}"),
        }
    }

    async fn finish_with_oauth(&self, flow_id: &str) -> Result<FinishedLogin, AuthError> {
        self.use_case
            .finish_login(
                flow_id,
                Credential::OAuthCallback("code=abc&state=csrf".to_string()),
            )
            .await
    }

    fn wait(&self) -> Waiting {
        let use_case = self.use_case.clone();
        let token = self.token.clone();
        tokio::spawn(async move { use_case.wait_for_session(&token).await })
    }

    async fn stored_login(&self) -> Option<LoginId> {
        self.account_repository
            .login_id_by_token(&self.token)
            .await
            .unwrap()
    }
}

fn bridge_metadata() -> BridgeMetadata {
    BridgeMetadata {
        name: "Tachyon".to_string(),
        client_uri: "https://bridge.example".to_string(),
        image_url: None,
        tos: None,
    }
}

fn flow_id(step: &Step) -> String {
    match step {
        Step::Authenticate { flow_id, .. } => flow_id.clone(),
        Step::VerifyDevice => panic!("the login is past authentication"),
    }
}

async fn still_waiting(waiting: &Waiting) -> bool {
    tokio::time::sleep(Duration::from_millis(20)).await;
    !waiting.is_finished()
}

async fn outcome(waiting: Waiting) -> Result<Arc<dyn BackendSession>, AuthError> {
    timeout(Duration::from_secs(2), waiting)
        .await
        .expect("the wait should have ended")
        .unwrap()
}

#[tokio::test]
async fn signing_in_with_a_verified_device_opens_the_session() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Verified]);

    let signed_in = fixture.sign_in().await.unwrap();

    assert!(matches!(signed_in, SignIn::Ready(_)));
    assert!(fixture.use_case.session(&fixture.token).is_some());
    assert_eq!(fixture.auth_service.restore_calls(), 1);
}

#[tokio::test]
async fn signing_in_with_an_unverified_device_asks_for_verification_and_withholds_the_session() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Unverified]);

    let step = fixture.sign_in_pending().await;

    assert!(matches!(step, Step::VerifyDevice));
    assert!(fixture.use_case.session(&fixture.token).is_none());
    assert!(fixture.use_case.has_login(&fixture.token));
}

#[tokio::test]
async fn the_wait_resolves_once_the_device_is_verified() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Unverified]);
    fixture.sign_in_pending().await;

    let waiting = fixture.wait();
    assert!(still_waiting(&waiting).await, "an unverified device must keep the caller waiting");

    fixture.session.verify();

    outcome(waiting).await.unwrap();
    assert!(fixture.use_case.session(&fixture.token).is_some());
    assert_eq!(fixture.auth_service.restore_calls(), 1);
}

#[tokio::test]
async fn two_concurrent_sign_ins_build_only_one_session() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Verified]);

    let (first, second) = tokio::join!(fixture.sign_in(), fixture.sign_in());

    assert!(matches!(first.unwrap(), SignIn::Ready(_)));
    assert!(matches!(second.unwrap(), SignIn::Ready(_)));
    assert_eq!(fixture.auth_service.restore_calls(), 1);
}

#[tokio::test]
async fn signing_in_without_a_stored_login_starts_an_interactive_one() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Verified]);

    let step = fixture.sign_in_pending().await;

    let Step::Authenticate { flow_id, prompt } = &step else {
        panic!("expected an authentication step, got {step:?}");
    };
    assert_eq!(flow_id, "csrf");
    assert!(matches!(prompt, InteractiveAuthStarted::OAuth { .. }));
    assert_eq!(fixture.auth_service.start_calls(), 1);
    assert_eq!(fixture.auth_service.restore_calls(), 0);
    assert_eq!(fixture.session.device_status_calls(), 0);
    assert!(fixture.use_case.session(&fixture.token).is_none());
    assert!(fixture.use_case.has_login(&fixture.token));
    assert!(matches!(
        fixture.use_case.prompt(flow_id),
        Some(InteractiveAuthStarted::OAuth { .. })
    ));
}

#[tokio::test]
async fn finishing_the_browser_login_binds_the_token_and_reports_the_device() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Unverified]);
    let flow_id = flow_id(&fixture.sign_in_pending().await);

    let finished = fixture.finish_with_oauth(&flow_id).await.unwrap();

    assert_eq!(finished.token, fixture.token);
    assert_eq!(finished.device_status, DeviceStatus::Unverified);
    assert_eq!(fixture.session.authenticate_calls(), 1);
    assert!(fixture.stored_login().await.is_some());
    assert!(fixture.use_case.has_login(&fixture.token));
    assert!(fixture.use_case.session(&fixture.token).is_none());
    assert!(fixture.use_case.prompt(&flow_id).is_none());
}

#[tokio::test]
async fn finishing_with_an_unknown_flow_is_refused() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Verified]);
    fixture.sign_in_pending().await;

    let refused = fixture.finish_with_oauth("not-a-flow").await;

    assert!(matches!(refused, Err(AuthError::LoginNotFound)));
    assert_eq!(fixture.session.authenticate_calls(), 0);
}

#[tokio::test]
async fn a_waiter_parked_on_the_browser_login_follows_through_to_the_session() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Unverified]);
    let flow_id = flow_id(&fixture.sign_in_pending().await);
    let waiting = fixture.wait();
    assert!(still_waiting(&waiting).await);

    fixture.finish_with_oauth(&flow_id).await.unwrap();
    assert!(still_waiting(&waiting).await, "the device is still unverified");

    fixture.session.verify();

    outcome(waiting).await.unwrap();
    assert!(fixture.use_case.session(&fixture.token).is_some());
}

#[tokio::test]
async fn finishing_a_login_whose_device_is_already_verified_makes_it_ready() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Verified]);
    let flow_id = flow_id(&fixture.sign_in_pending().await);
    let waiting = fixture.wait();

    let finished = fixture.finish_with_oauth(&flow_id).await.unwrap();

    assert_eq!(finished.device_status, DeviceStatus::Verified);
    outcome(waiting).await.unwrap();
    assert!(fixture.use_case.session(&fixture.token).is_some());
}

#[tokio::test]
async fn abandoning_releases_a_waiter_parked_on_the_browser_login() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Verified]);
    fixture.sign_in_pending().await;
    let waiting = fixture.wait();
    assert!(still_waiting(&waiting).await);

    fixture.use_case.abandon(&fixture.token).await.unwrap();

    assert!(matches!(outcome(waiting).await, Err(AuthError::LoginNotFound)));
    assert_eq!(fixture.session.discard_calls(), 1, "nothing to restore, so nothing to keep");
    assert_eq!(fixture.session.close_calls(), 0);
    assert_eq!(fixture.session.log_out_calls(), 0, "only delete credentials logs a device out");
    assert!(!fixture.use_case.has_login(&fixture.token));
}

#[tokio::test]
async fn abandoning_releases_a_waiter_parked_on_device_verification() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Unverified]);
    fixture.sign_in_pending().await;
    let waiting = fixture.wait();
    assert!(still_waiting(&waiting).await);

    fixture.use_case.abandon(&fixture.token).await.unwrap();

    assert!(outcome(waiting).await.is_err());
    assert_eq!(fixture.session.close_calls(), 1, "an authenticated login can be restored");
    assert_eq!(fixture.session.discard_calls(), 0);
    assert!(!fixture.use_case.has_login(&fixture.token));
}

#[tokio::test]
async fn a_second_sign_in_replaces_a_pending_login() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Verified]);
    fixture.sign_in_pending().await;

    fixture.sign_in_pending().await;

    assert_eq!(fixture.session.discard_calls(), 1);
    assert_eq!(fixture.auth_service.start_calls(), 2);
    assert!(fixture.use_case.has_login(&fixture.token));
}

#[tokio::test]
async fn a_sign_in_whose_device_status_cannot_be_read_leaves_nothing_behind() {
    let fixture = Fixture::new(FakeBackendSession::with_unreachable_device_status(), true);

    let error = fixture.sign_in().await.err().unwrap();

    assert!(matches!(error, AuthError::BackendError(_)));
    assert!(!fixture.use_case.has_login(&fixture.token));
    assert_eq!(fixture.session.close_calls(), 1);

    fixture.sign_in().await.err().unwrap();
    assert_eq!(fixture.auth_service.restore_calls(), 2);
}

#[tokio::test]
async fn abandoning_a_login_that_is_not_there_is_fine() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Verified]);

    fixture.use_case.abandon(&fixture.token).await.unwrap();

    assert_eq!(fixture.session.close_calls(), 0);
}

#[tokio::test]
async fn abandoning_a_ready_login_closes_and_forgets_its_session() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Verified]);
    fixture.sign_in().await.unwrap();

    fixture.use_case.abandon(&fixture.token).await.unwrap();

    assert_eq!(fixture.session.close_calls(), 1);
    assert_eq!(fixture.session.log_out_calls(), 0, "the device stays so a restore works");
    assert!(fixture.use_case.session(&fixture.token).is_none());
    assert!(fixture.stored_login().await.is_some(), "the store keeps the login for next time");
}

#[tokio::test]
async fn a_pending_sign_in_says_where_the_browser_must_go() {
    let interactive = Fixture::without_stored_login([DeviceStatus::Verified]);
    let verification = Fixture::with_stored_login([DeviceStatus::Unverified]);

    let (_, login_url) = interactive.sign_in_pending_with_url().await;
    let (_, confirm_url) = verification.sign_in_pending_with_url().await;

    assert_eq!(login_url, "https://bridge.example/login/start?flow=csrf");
    assert_eq!(
        confirm_url,
        "https://bridge.example/confirm_device?t=tachyon-token"
    );
}

#[tokio::test]
async fn finishing_with_a_password_binds_the_token_like_the_browser_callback() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Verified]);
    let flow_id = flow_id(&fixture.sign_in_pending().await);

    let finished = fixture
        .use_case
        .finish_login(&flow_id, Credential::Password(Password::new("hunter2")))
        .await
        .unwrap();

    assert_eq!(finished.device_status, DeviceStatus::Verified);
    assert_eq!(fixture.session.authenticate_calls(), 1);
    assert!(fixture.stored_login().await.is_some());
    assert!(fixture.use_case.session(&fixture.token).is_some());
}

#[tokio::test]
async fn a_rejected_credential_leaves_the_login_pending_for_another_try() {
    let fixture = Fixture::new(FakeBackendSession::rejecting_credentials(), false);
    let flow_id = flow_id(&fixture.sign_in_pending().await);
    let waiting = fixture.wait();

    let refused = fixture
        .use_case
        .finish_login(&flow_id, Credential::Password(Password::new("wrong")))
        .await;

    assert!(matches!(refused, Err(AuthError::BackendError(_))));
    assert!(fixture.use_case.has_login(&fixture.token));
    assert!(fixture.use_case.prompt(&flow_id).is_some());
    assert_eq!(fixture.session.close_calls(), 0);
    assert!(fixture.stored_login().await.is_none());
    assert!(still_waiting(&waiting).await, "the client keeps waiting for the retry");
}

#[tokio::test]
async fn forgetting_a_live_login_discards_it_and_drops_its_row() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Verified]);
    fixture.sign_in().await.unwrap();

    fixture.use_case.forget(&fixture.token).await.unwrap();

    assert_eq!(fixture.session.log_out_calls(), 1);
    assert_eq!(fixture.session.discard_calls(), 1);
    assert!(fixture.use_case.session(&fixture.token).is_none());
    assert!(fixture.stored_login().await.is_none());
}

#[tokio::test]
async fn forgetting_a_stored_login_reaches_the_backend_and_drops_its_row() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Verified]);

    fixture.use_case.forget(&fixture.token).await.unwrap();

    assert_eq!(fixture.session.log_out_calls(), 1, "rebuilt from the store to end its device");
    assert_eq!(fixture.session.discard_calls(), 1);
    assert_eq!(fixture.auth_service.forgotten(), vec![LoginId::new("login-1")]);
    assert!(fixture.stored_login().await.is_none());
}

#[tokio::test]
async fn forgetting_a_token_with_no_login_is_fine() {
    let fixture = Fixture::without_stored_login([DeviceStatus::Verified]);

    fixture.use_case.forget(&fixture.token).await.unwrap();

    assert!(fixture.auth_service.forgotten().is_empty());
}

#[tokio::test]
async fn a_login_the_backend_no_longer_honours_is_forgotten_and_started_afresh() {
    let fixture = Fixture::with_stored_login([DeviceStatus::Verified]);
    fixture.auth_service.fail_restores_as_logged_out();

    let step = fixture.sign_in_pending().await;

    assert!(matches!(step, Step::Authenticate { .. }));
    assert_eq!(fixture.auth_service.forgotten(), vec![LoginId::new("login-1")]);
    assert!(fixture.stored_login().await.is_none(), "the dead login's row is gone");
    assert_eq!(fixture.auth_service.start_calls(), 1);
}

#[tokio::test]
async fn sweeping_forgets_unbound_logins_and_prunes_the_backend_to_the_rest() {
    let store = AccountRepositoryInMem::with_login(&TachyonToken::new("tachyon-token"), &LoginId::new("login-1"))
        .with_unbound_login(&LoginId::new("leftover"));
    let fixture = Fixture::over(FakeBackendSession::new([DeviceStatus::Verified]), store);

    fixture.use_case.sweep().await.unwrap();

    assert_eq!(fixture.auth_service.forgotten(), vec![LoginId::new("leftover")]);
    assert_eq!(fixture.auth_service.swept_keeping(), Some(vec![LoginId::new("login-1")]));
    assert!(fixture.stored_login().await.is_some(), "a bound login is kept");
}
