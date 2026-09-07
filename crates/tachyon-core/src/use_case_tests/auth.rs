use crate::application::auth_use_case::{AuthUseCase, LoginOutcome};
use crate::application::error::{AuthError, ReadinessError};
use crate::application::ports::SessionRepository;
use crate::application::test_support::{
    FakeAccountRepository, FakeAuthService, FakeBackendSession,
};
use crate::domain::auth::{BridgeMetadata, InteractiveAuthStarted, Readiness, TachyonToken};
use crate::domain::ids::{LoginId, UserId};
use crate::domain::verification::DeviceStatus;
use crate::infrastructure::repository::SessionRepositoryInMem;
use std::sync::Arc;

struct Fixture {
    use_case: AuthUseCase,
    session_repository: Arc<SessionRepositoryInMem>,
    auth_service: Arc<FakeAuthService>,
    session: Arc<FakeBackendSession>,
    token: TachyonToken,
    login_id: LoginId,
}

impl Fixture {
    fn with_device_statuses(statuses: impl IntoIterator<Item = DeviceStatus>) -> Self {
        let token = TachyonToken::new("tachyon-token");
        let login_id = LoginId::new("login-1");
        let session = FakeBackendSession::new(statuses);
        let auth_service = FakeAuthService::new(session.clone());
        let session_repository = Arc::new(SessionRepositoryInMem::default());
        let use_case = AuthUseCase::new(
            FakeAccountRepository::with_login(&token, &login_id),
            session_repository.clone(),
            auth_service.clone(),
            "https://bridge.example/callback".to_string(),
        );

        Self {
            use_case,
            session_repository,
            auth_service,
            session,
            token,
            login_id,
        }
    }

    fn readiness(&self) -> Option<Readiness> {
        self.session_repository
            .get(&self.login_id)
            .map(|entry| entry.readiness)
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

#[tokio::test]
async fn restoring_a_verified_device_opens_the_session() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);

    let outcome = fixture.use_case.restore(&fixture.token).await.unwrap();

    assert!(matches!(outcome, LoginOutcome::SessionOpened { .. }));
    assert_eq!(fixture.readiness(), Some(Readiness::Ready));
    assert!(fixture.session_repository.get_ready(&fixture.login_id).is_some());
}

#[tokio::test]
async fn restoring_an_unverified_device_asks_for_verification_and_withholds_the_session() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Unverified]);

    let outcome = fixture.use_case.restore(&fixture.token).await.unwrap();

    assert!(matches!(
        outcome,
        LoginOutcome::DeviceVerificationRequired { .. }
    ));
    assert_eq!(fixture.readiness(), Some(Readiness::VerificationNeeded));
    assert!(fixture.session_repository.get_ready(&fixture.login_id).is_none());
}

#[tokio::test]
async fn restoring_again_after_the_device_is_verified_opens_the_same_session() {
    let fixture =
        Fixture::with_device_statuses([DeviceStatus::Unverified, DeviceStatus::Verified]);

    let first = fixture.use_case.restore(&fixture.token).await.unwrap();
    assert!(matches!(
        first,
        LoginOutcome::DeviceVerificationRequired { .. }
    ));

    let second = fixture.use_case.restore(&fixture.token).await.unwrap();

    assert!(matches!(second, LoginOutcome::SessionOpened { .. }));
    assert_eq!(fixture.readiness(), Some(Readiness::Ready));
    assert_eq!(fixture.session.device_status_calls(), 2);
    assert_eq!(fixture.auth_service.restore_calls(), 1);
}

#[tokio::test]
async fn two_concurrent_restores_build_only_one_session() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);

    let (first, second) = tokio::join!(
        fixture.use_case.restore(&fixture.token),
        fixture.use_case.restore(&fixture.token)
    );

    assert!(matches!(
        first.unwrap(),
        LoginOutcome::SessionOpened { .. }
    ));
    assert!(matches!(
        second.unwrap(),
        LoginOutcome::SessionOpened { .. }
    ));
    assert_eq!(fixture.auth_service.restore_calls(), 1);
}

#[tokio::test]
async fn restoring_a_token_with_no_login_reports_missing_credentials() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);

    let outcome = fixture
        .use_case
        .restore(&TachyonToken::new("unknown-token"))
        .await;

    assert!(matches!(
        outcome,
        Err(AuthError::BackendCredentialsNotInStore)
    ));
}

#[tokio::test]
async fn abandoning_a_login_that_is_not_there_is_fine() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);

    assert!(fixture.use_case.abandon_login(&fixture.login_id).await.is_ok());
    assert_eq!(fixture.session.close_calls(), 0);
}

#[tokio::test]
async fn abandoning_a_login_closes_and_forgets_its_session() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);
    fixture.use_case.restore(&fixture.token).await.unwrap();

    fixture
        .use_case
        .abandon_login(&fixture.login_id)
        .await
        .unwrap();

    assert_eq!(fixture.session.close_calls(), 1);
    assert!(fixture.session_repository.get(&fixture.login_id).is_none());
}

#[tokio::test]
async fn finishing_an_interactive_login_settles_the_readiness_it_ends_up_in() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Unverified]);
    fixture.session_repository.insert(
        fixture.login_id.clone(),
        fixture.session.clone(),
        Readiness::AuthNeeded,
    );

    let outcome = fixture
        .use_case
        .finish_interactive_login(&fixture.login_id, "code=abc&state=xyz")
        .await
        .unwrap();

    assert!(matches!(
        outcome,
        LoginOutcome::DeviceVerificationRequired { .. }
    ));
    assert_eq!(fixture.session.finish_calls(), 1);
    assert_eq!(fixture.readiness(), Some(Readiness::VerificationNeeded));
}

#[tokio::test]
async fn finishing_an_interactive_login_without_a_pending_one_is_refused() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);

    let outcome = fixture
        .use_case
        .finish_interactive_login(&fixture.login_id, "code=abc")
        .await;

    assert!(matches!(outcome, Err(AuthError::LoginNotFound)));
    assert_eq!(fixture.session.finish_calls(), 0);
}

#[tokio::test]
async fn readiness_cannot_be_walked_back() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);
    fixture.use_case.restore(&fixture.token).await.unwrap();

    let refused = fixture
        .session_repository
        .set_readiness(&fixture.login_id, Readiness::VerificationNeeded);

    assert!(matches!(
        refused,
        Err(ReadinessError::Backwards {
            from: Readiness::Ready,
            to: Readiness::VerificationNeeded
        })
    ));
    assert_eq!(fixture.readiness(), Some(Readiness::Ready));
}

#[tokio::test]
async fn setting_the_readiness_of_an_unknown_login_is_refused() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);

    let refused = fixture
        .session_repository
        .set_readiness(&fixture.login_id, Readiness::Ready);

    assert!(matches!(refused, Err(ReadinessError::NotFound)));
}

#[tokio::test]
async fn starting_an_interactive_login_parks_the_session_until_it_authenticates() {
    let fixture = Fixture::with_device_statuses([DeviceStatus::Verified]);

    let start = fixture
        .use_case
        .start_interactive_login(
            "example.org",
            UserId::new("@someone:example.org"),
            &bridge_metadata(),
        )
        .await
        .unwrap();

    assert_eq!(fixture.auth_service.start_calls(), 1);
    assert!(matches!(
        start.prompt,
        InteractiveAuthStarted::OAuth { .. }
    ));
    assert_eq!(
        fixture
            .session_repository
            .get(&start.login_id)
            .map(|entry| entry.readiness),
        Some(Readiness::AuthNeeded)
    );
    assert!(fixture.session_repository.get_ready(&start.login_id).is_none());
    assert_eq!(fixture.session.device_status_calls(), 0);
}
