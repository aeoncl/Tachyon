use crate::application::device_verification_use_case::DeviceVerificationUseCase;
use crate::application::error::VerificationError;
use crate::application::ports::SessionRepository;
use crate::application::test_support::{FakeAccountRepository, FakeBackendSession};
use crate::domain::auth::{Readiness, TachyonToken};
use crate::domain::ids::{DeviceId, LoginId};
use crate::domain::verification::{
    DeviceStatus, RecoveryKey, VerificationAction, VerificationFlowState,
};
use crate::infrastructure::repository::SessionRepositoryInMem;
use std::sync::Arc;

struct Fixture {
    use_case: DeviceVerificationUseCase,
    session: Arc<FakeBackendSession>,
    token: TachyonToken,
}

impl Fixture {
    fn new(session: Arc<FakeBackendSession>, readiness: Option<Readiness>) -> Self {
        let token = TachyonToken::new("tachyon-token");
        let login_id = LoginId::new("login-1");
        let session_repository = Arc::new(SessionRepositoryInMem::default());
        if let Some(readiness) = readiness {
            session_repository.insert(login_id.clone(), session.clone(), readiness);
        }

        Self {
            use_case: DeviceVerificationUseCase::new(
                FakeAccountRepository::with_login(&token, &login_id),
                session_repository,
            ),
            session,
            token,
        }
    }

    fn unverified(statuses: impl IntoIterator<Item = DeviceStatus>) -> Self {
        Self::new(
            FakeBackendSession::new(statuses),
            Some(Readiness::VerificationNeeded),
        )
    }
}

#[tokio::test]
async fn a_recovery_key_that_does_not_verify_the_device_is_reported_as_still_unverified() {
    let fixture = Fixture::unverified([DeviceStatus::Unverified]);

    let outcome = fixture
        .use_case
        .recover(&fixture.token, &RecoveryKey::new("not-the-key"))
        .await;

    assert!(matches!(outcome, Err(VerificationError::StillUnverified)));
}

#[tokio::test]
async fn a_recovery_key_that_verifies_the_device_succeeds() {
    let fixture = Fixture::unverified([DeviceStatus::Verified]);

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
    let fixture = Fixture::new(
        FakeBackendSession::new([DeviceStatus::Unverified]),
        Some(Readiness::Ready),
    );

    let status = fixture.use_case.status(&fixture.token).await.unwrap();

    assert_eq!(status, DeviceStatus::Verified);
    assert_eq!(fixture.session.device_status_calls(), 0);
}

#[tokio::test]
async fn a_ready_login_still_reports_the_flow_that_verified_it() {
    let fixture = Fixture::new(
        FakeBackendSession::new([DeviceStatus::Verified])
            .with_verification_state(VerificationFlowState::Done),
        Some(Readiness::Ready),
    );

    let state = fixture
        .use_case
        .verification_state(&fixture.token)
        .await
        .unwrap();

    assert_eq!(state, VerificationFlowState::Done);
}

#[tokio::test]
async fn a_ready_login_refuses_every_call_that_would_change_its_device_trust() {
    let fixture = Fixture::new(
        FakeBackendSession::new([DeviceStatus::Verified]),
        Some(Readiness::Ready),
    );

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
    let fixture = Fixture::new(
        FakeBackendSession::new([DeviceStatus::Unverified]),
        Some(Readiness::AuthNeeded),
    );

    let outcome = fixture.use_case.status(&fixture.token).await;

    assert!(matches!(outcome, Err(VerificationError::NotAuthenticated)));
}

#[tokio::test]
async fn a_token_with_no_session_is_not_found() {
    let fixture = Fixture::new(FakeBackendSession::new([DeviceStatus::Unverified]), None);

    let known_token_no_session = fixture.use_case.status(&fixture.token).await;
    let unknown_token = fixture
        .use_case
        .status(&TachyonToken::new("unknown-token"))
        .await;

    assert!(matches!(
        known_token_no_session,
        Err(VerificationError::LoginNotFound)
    ));
    assert!(matches!(unknown_token, Err(VerificationError::LoginNotFound)));
}

#[tokio::test]
async fn asking_for_a_flow_that_was_never_started_reports_no_verification_in_progress() {
    let fixture = Fixture::unverified([DeviceStatus::Unverified]);

    let outcome = fixture.use_case.verification_state(&fixture.token).await;

    assert!(matches!(
        outcome,
        Err(VerificationError::NoVerificationInProgress)
    ));
}
