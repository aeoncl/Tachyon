use std::str::FromStr;
use axum::extract::{Multipart, State};
use axum::response::Html;
use maud::html;
use crate::tachyon::alert::{AlertError, AlertNotify, AlertSuccess};
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::{layout, Params};

pub async fn get_verify(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Html<String> {

    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let _notification = tachyon_client.alerts().get(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    let recover_enabled = matrix_client
        .encryption()
        .secret_storage()
        .is_enabled()
        .await
        .unwrap();

    let content = html! {
        h2 { "Verify your device" }
        @if recover_enabled {
            p { "You can use your recovery key to verify this device." }
            form method="POST" ic-post-to="/tachyon/verify_device/recovery_key" ic-target=".content" enctype="multipart/form-data" {
                input type="file" name="recovery_key" accept=".txt" required;
                input type="hidden" name="notification_id" value=(notification_id) required;
                button type="submit" { "Verify Device" }
            }
        } @else {
            p { "It looks like you don't have a recovery key set up." }
            button type="button" ic-post-to="/tachyon/verify_device" ic-target=".content" { "Reset Identity" }
        }
    };

    Html(layout::tachyon_page(content).into_string())
}

pub async fn verify_recovery_key(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    mut multipart: Multipart,
) -> Html<String> {
    let mut notification_id = None;
    let mut recovery_key = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("notification_id") => {
                notification_id = Some(i32::from_str(&field.text().await.unwrap()).unwrap());
            }
            Some("recovery_key") => {
                recovery_key = Some(field.text().await.unwrap());
            }
            _ => {}
        }
    }

    let notification_id = notification_id.unwrap();
    let recovery_key = recovery_key.unwrap().replace(" ", "");

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let (_id, mut alert) = tachyon_client.alerts().remove(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    println!("Recovering device with recovery key: {}", recovery_key);

    let store = matrix_client
        .encryption()
        .secret_storage()
        .open_secret_store(recovery_key.trim())
        .await
        .unwrap();
    store.import_secrets().await.unwrap();

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

    let successful = status.is_complete();

    if successful {
        alert.notify_success(AlertSuccess::Unit);
    } else {
        alert.notify_failure(AlertError::from(anyhow::anyhow!("Failed to verify device")));
    }

    let page = html! {
        h2 { "Verification Result" }
        @if successful {
            p { "Your device is now verified!" }
        } @else {
            p { "Your device could not be verified :(" }
        }
    };

    Html(page.into_string())
}