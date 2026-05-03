use std::str::FromStr;
use axum::body::Body;
use axum::extract::{State, Path};
use axum::http::{Response, StatusCode};
use axum::response::IntoResponse;
use matrix_sdk::ruma::OwnedUserId;
use crate::tachyon::global::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::Params;

pub(crate) async fn post_sas_v1_action(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    Path(action): Path<String>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> impl IntoResponse {
    let notification_id_raw = form_data.get("notification_id").map(|s| s.as_str()).unwrap();
    let notification_id = i32::from_str(notification_id_raw).unwrap();

    let flow_id = form_data.get("flow_id").map(|s| s.as_str()).unwrap();

    let user_id_raw = form_data.get("user_id").map(|s| s.as_str()).unwrap();
    let user_id = OwnedUserId::from_str(user_id_raw).unwrap();

    let client = state.tachyon_clients().get(&token).unwrap().matrix_client().clone();

    let verification = client.encryption().get_verification(&user_id, flow_id).await.unwrap();

    let sas = verification.sas().unwrap();

    match action.to_lowercase().as_str() {
        "confirm" => sas.confirm().await.unwrap(),
        "mismatch" => sas.mismatch().await.unwrap(),
        "cancel" => sas.cancel().await.unwrap(),
        "accept" => sas.accept().await.unwrap(),
        _ => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("Invalid action: {}", action)))
                .unwrap();
        }
    }

    let return_url = format!("/tachyon/verification?notification_id={}&flow_id={}&user_id={}", notification_id, flow_id, &user_id);

    Response::builder()
        .status(StatusCode::OK)
        .header("X-IC-Redirect", return_url)
        .body(Body::empty())
        .unwrap()
}