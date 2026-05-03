use std::str::FromStr;
use axum::extract::State;
use axum::response::Html;
use matrix_sdk::ruma::api::client::uiaa::{AuthData, Password, UserIdentifier};
use maud::html;
use crate::matrix::cross_signing::check_device_is_crossed_signed;
use crate::tachyon::alert::{AlertError, AlertNotify, AlertSuccess};
use crate::tachyon::global::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::Params;

pub async fn post_reset_identity(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {

    let notification_id_raw = form_data.get("notification_id").map(|s| s.as_str()).unwrap();
    let notification_id = i32::from_str(notification_id_raw).unwrap();

    let password = form_data.get("password").map(|s| s.as_str()).unwrap();


    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let (_id, mut alert) = tachyon_client.alerts().remove(&notification_id).unwrap();
    let matrix_client = tachyon_client.matrix_client().clone();

    let own_user = matrix_client.user_id().unwrap();

    let handle = matrix_client.encryption().recovery().reset_identity().await.unwrap();
    if let Some(handle) = handle {
        handle.reset(Some(AuthData::Password(Password::new(UserIdentifier::UserIdOrLocalpart(own_user.to_string()), password.to_string())))).await.unwrap();
    }

    let secret_storage_key = matrix_client.encryption()
        .recovery()
        .enable()
        .await.unwrap();


    let status = matrix_client
        .encryption()
        .cross_signing_status()
        .await
        .expect("We should be able to check our cross-signing status");

    matrix_client
        .encryption()
        .get_own_device()
        .await
        .unwrap()
        .unwrap()
        .verify()
        .await
        .unwrap();

    let successful = status.is_complete() && check_device_is_crossed_signed(&matrix_client).await.unwrap();

    if successful {
        alert.notify_success(AlertSuccess::Unit);
    } else {
        alert.notify_failure(AlertError::from(anyhow::anyhow!("Failed to verify device")));
    }

    let page = html! {
        h2 { "Verification Result" }
        @if successful {
            p { "Your device is now verified!" }
            p { "Here is your recovery key, store it somewhere safe:" }
            pre { (secret_storage_key.to_string())}
        } @else {
            p { "Your device could not be verified :(" }
        }
    };

    Html(page.into_string())
}

pub async fn get_reset_identity(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Html<String> {

    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let page = html! {
        form action="/tachyon/confirm_device/reset_identity" method="POST" ic-post-to="/tachyon/confirm_device/reset_identity" ic-target=".content" {
            div id="error-message" style="display:none;" {}
            input type="password" name="password" id="password" {}
            input type="hidden" name="notification_id" value=(notification_id) {}
            button type="submit" class="btn btn-primary" {
                span class="btn-shine" {}
                span class="btn-label" { "Reset Identity" }
            }
        }
    };

    Html(page.into_string())

}