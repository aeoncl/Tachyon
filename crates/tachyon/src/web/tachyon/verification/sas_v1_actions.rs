use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::layout::error_fragment;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Response, StatusCode};
use axum::response::{Html, IntoResponse};
use tachyon_core::domain::auth::BridgeLinkToken;
use tachyon_core::domain::verification::VerificationAction;

pub(crate) async fn post_sas_v1_action(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    Path(action): Path<String>,
) -> Response<Body> {
    let action = match action.to_lowercase().as_str() {
        "confirm" => VerificationAction::Confirm,
        "mismatch" => VerificationAction::Mismatch,
        "cancel" => VerificationAction::Cancel,
        other => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("Invalid action: {}", other)))
                .unwrap();
        }
    };

    let use_case = state.app_state().device_verification_use_case();
    if let Err(e) = use_case
        .verification_action(&BridgeLinkToken::new(&token), action)
        .await
    {
        return Html(error_fragment(&e.to_string()).into_string()).into_response();
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("X-IC-Redirect", "/tachyon/verification")
        .body(Body::empty())
        .unwrap()
}
