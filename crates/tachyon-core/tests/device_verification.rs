//! Device verification over the real `Logins`, with the `tachyon_testkit` fakes standing in
//! for the backend and the store. Logins are put into each state through `AuthUseCase`, the
//! way a bridge would.

use std::sync::Arc;
use tachyon_core::application::auth_use_case::AuthUseCase;
use tachyon_core::application::web_urls::WebUrls;
use tachyon_core::application::device_verification_use_case::DeviceVerificationUseCase;
use tachyon_core::application::error::VerificationError;
use tachyon_core::application::logins::Logins;
use tachyon_core::domain::auth::{BridgeMetadata, BridgeLinkToken};
use tachyon_core::domain::ids::{DeviceId, LoginId, UserId};
use tachyon_core::domain::verification::{
    DeviceStatus, RecoveryKey, VerificationAction, VerificationFlowState,
};
use tachyon_testkit::fakes::{FakeAuthService, FakeBackendSession};
use tachyon_testkit::repositories::AccountRepositoryInMem;

#[derive(PartialEq)]
enum LoginState {
    /// No login for the token at all.
    Absent,
    /// An interactive login the browser has not finished.
    Authenticating,
    /// Authenticated, device untrusted.
    Verifying,
    /// Signed in and trusted.
    Ready,
}

struct Fixture {
    use_case: DeviceVerificationUseCase,
    session: Arc<FakeBackendSession>,
    token: BridgeLinkToken,
}

impl Fixture {
    /// `statuses` is the session's device status script. `sign_in` consumes the first entry
    /// for every state past `Authenticating`.
    async fn new(statuses: impl IntoIterator<Item = DeviceStatus>, state: LoginState) -> Self {
        let session = FakeBackendSession::new(statuses);
        let token = BridgeLinkToken::new("tachyon-token");
        let logins = Arc::new(Logins::default());
        let account_repository = match state {
            LoginState::Absent | LoginState::Authenticating => {
                Arc::new(AccountRepositoryInMem::default())
            }
            LoginState::Verifying | LoginState::Ready => {
                AccountRepositoryInMem::with_login(&token, &LoginId::new("login-1"))
            }
        };
        let auth = AuthUseCase::new(
            account_repository,
            logins.clone(),
            FakeAuthService::handing_out(session.clone()),
            WebUrls::new("https://bridge.example"),
        );

        if state != LoginState::Absent {
            auth.sign_in_or_restore_login(
                &token,
                "example.org",
                UserId::new("@someone:example.org"),
                &BridgeMetadata {
                    name: "Tachyon".to_string(),
                    client_uri: "https://bridge.example".to_string(),
                    image_url: None,
                    tos: None,
                },
            )
            .await
            .unwrap();
        }

        Self {
            use_case: DeviceVerificationUseCase::new(logins),
            session,
            token,
        }
    }

    async fn verifying(statuses: impl IntoIterator<Item = DeviceStatus>) -> Self {
        Self::new(statuses, LoginState::Verifying).await
    }

    async fn ready() -> Self {
        Self::new(
            [DeviceStatus::Verified, DeviceStatus::Unverified],
            LoginState::Ready,
        )
        .await
    }
}

#[tokio::test]
async fn a_recovery_key_that_does_not_verify_the_device_is_reported_as_still_unverified() {
    let fixture = Fixture::verifying([DeviceStatus::Unverified]).await;

    let outcome = fixture
        .use_case
        .recover(&fixture.token, &RecoveryKey::new("not-the-key"))
        .await;

    assert!(matches!(outcome, Err(VerificationError::StillUnverified)));
}

#[tokio::test]
async fn a_recovery_key_that_verifies_the_device_succeeds() {
    let fixture = Fixture::verifying([DeviceStatus::Unverified, DeviceStatus::Verified]).await;

    assert!(
        fixture
            .use_case
            .recover(&fixture.token, &RecoveryKey::new("the-key"))
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn a_ready_login_is_verified_without_asking_the_backend() {
    let fixture = Fixture::ready().await;
    let asked_before = fixture.session.device_status_calls();

    let status = fixture.use_case.status(&fixture.token).await.unwrap();

    assert_eq!(status, DeviceStatus::Verified);
    assert_eq!(fixture.session.device_status_calls(), asked_before);
}

#[tokio::test]
async fn a_ready_login_still_reports_the_flow_that_verified_it() {
    let fixture = Fixture::ready().await;
    fixture
        .session
        .clone()
        .with_verification_state(VerificationFlowState::Done);

    let state = fixture
        .use_case
        .verification_state(&fixture.token)
        .await
        .unwrap();

    assert_eq!(state, VerificationFlowState::Done);
}

#[tokio::test]
async fn a_ready_login_refuses_every_call_that_would_change_its_device_trust() {
    let fixture = Fixture::ready().await;

    let recover = fixture
        .use_case
        .recover(&fixture.token, &RecoveryKey::new("the-key"))
        .await;
    let start = fixture
        .use_case
        .start_device_verification(&fixture.token, &DeviceId::new("OTHERDEVICE"))
        .await;
    let action = fixture
        .use_case
        .verification_action(&fixture.token, VerificationAction::Confirm)
        .await;
    let reset = fixture.use_case.reset_identity(&fixture.token, None).await;

    assert!(matches!(recover, Err(VerificationError::AlreadyVerified)));
    assert!(matches!(start, Err(VerificationError::AlreadyVerified)));
    assert!(matches!(action, Err(VerificationError::AlreadyVerified)));
    assert!(matches!(reset, Err(VerificationError::AlreadyVerified)));
}

#[tokio::test]
async fn a_login_that_has_not_authenticated_yet_cannot_be_verified() {
    let fixture = Fixture::new([DeviceStatus::Unverified], LoginState::Authenticating).await;

    let outcome = fixture.use_case.status(&fixture.token).await;

    assert!(matches!(outcome, Err(VerificationError::NotAuthenticated)));
}

#[tokio::test]
async fn a_token_with_no_login_is_not_found() {
    let fixture = Fixture::new([DeviceStatus::Unverified], LoginState::Absent).await;

    let outcome = fixture.use_case.status(&fixture.token).await;

    assert!(matches!(outcome, Err(VerificationError::LoginNotFound)));
}

#[tokio::test]
async fn asking_for_a_flow_that_was_never_started_reports_no_verification_in_progress() {
    let fixture = Fixture::verifying([DeviceStatus::Unverified]).await;

    let outcome = fixture.use_case.verification_state(&fixture.token).await;

    assert!(matches!(
        outcome,
        Err(VerificationError::NoVerificationInProgress)
    ));
}
