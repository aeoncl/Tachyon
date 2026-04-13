use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::Params;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::response::{Html, IntoResponse};
use matrix_sdk::encryption::identities::Device;
use matrix_sdk::ruma::{device_id, DeviceId};
use maud::{html, Markup};
use std::str::FromStr;
use matrix_sdk::encryption::verification::{VerificationRequest, VerificationRequestState};
use matrix_sdk::ruma::events::key::verification::VerificationMethod;

pub async fn get_other_device(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Html<String> {
    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let _notification = tachyon_client.alerts().get(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    let has_devices_to_confirm_with = matrix_client.encryption().has_devices_to_verify_against().await.unwrap();

    let devices = matrix_client.encryption().get_user_devices(matrix_client.user_id().unwrap()).await.unwrap();

    let verifiable_devices = devices.devices().filter(|device| {
        device.is_cross_signed_by_owner()
            && device.curve25519_key().is_some()
            && !device.is_dehydrated()
    }).collect::<Vec<_>>();

    Html(restore_device_content(notification_id, has_devices_to_confirm_with, &verifiable_devices).into_string())
}

fn restore_device_content(notification_id: i32, has_devices_to_confirm_with: bool, devices: &[Device]) -> Markup {

    //TODO Filter devices that are not signed, showed them greyed out or something
    //TODO Show a message if there are no devices to confirm with

    html! {
        div class="container" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
            tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Restore with another device" }
                        p { "Please choose the device you want to use to start the restore process." }
                        p { "You will need to compare a bunch of emojis on both devices and check that they match !" }
                    }
                }
            }

            form action="/tachyon/confirm_device/other_device" method="POST" ic-post-to="/tachyon/confirm_device/other_device" ic-target=".content" {
                div id="error-message" style="display:none;" {}

                @for device in devices {
                  input type="radio" name="device" id=(device.device_id()) value=(device.device_id()) { (device.display_name().unwrap_or(device.device_id().as_str())) }
                }

                input type="hidden" id="recovery_key_full" name="recovery_key_full";
                input type="hidden" id="notification_id" name="notification_id" value=(notification_id);

                br {}

                button type="submit" class="btn btn-primary" {
                    span class="btn-shine" {}
                    span class="btn-label" { "Restore this device" }
                }
            }
        }
    }
}


pub async fn post_other_device(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> impl IntoResponse  {


    let notification_id_raw = form_data.get("notification_id").map(|s| s.as_str()).unwrap();
    let notification_id = i32::from_str(notification_id_raw).unwrap();

    let device_id_raw = form_data.get("device").map(|s| s.as_str()).unwrap();
    let device_id = device_id!(device_id_raw);

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let _notification = tachyon_client.alerts().get(&notification_id).unwrap();

    let matrix_client = state.matrix_clients().get(&token).unwrap();
    let user_id = matrix_client.user_id().unwrap();

    let device = matrix_client.encryption().get_device(user_id, device_id).await.unwrap().unwrap();

    let verification = device.request_verification_with_methods(vec![VerificationMethod::SasV1]).await.unwrap();


    let flow_id = verification.flow_id().to_string();
    state.pending_verification_requests().insert(flow_id.clone(), verification);


    let return_url = format!("/tachyon/verification?notification_id={}&flow_id={}&user_id={}", notification_id, flow_id, user_id);

    Response::builder()
        .status(StatusCode::OK)
        .header("X-IC-Redirect", return_url)
        .body(Body::empty())
        .unwrap()

}