use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::layout::error_fragment;
use crate::web::tachyon::Params;
use axum::body::Body;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use maud::{html, Markup};
use tachyon_core::domain::auth::TachyonToken;
use tachyon_core::domain::ids::DeviceId;
use tachyon_core::domain::verification::DeviceSummary;

pub async fn get_other_device(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
) -> Html<String> {
    let use_case = state.app_state().device_verification_use_case();

    let content = match use_case.options(&TachyonToken::new(&token)).await {
        Ok(options) => choose_device_content(&options.devices),
        Err(e) => error_fragment(&e.to_string()),
    };

    Html(content.into_string())
}

pub async fn post_other_device(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Response {
    let Some(device) = form_data.get("device").filter(|id| !id.is_empty()) else {
        return Html(error_fragment("Please choose a device to confirm with.").into_string())
            .into_response();
    };

    let use_case = state.app_state().device_verification_use_case();
    if let Err(e) = use_case
        .start_device_verification(&TachyonToken::new(&token), &DeviceId::new(device))
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

fn choose_device_content(devices: &[DeviceSummary]) -> Markup {
    html! {
        div class="container" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
            tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Restore with another device" }
                        @if devices.is_empty() {
                            p { "None of your other devices can confirm this one." }
                        } @else {
                            p { "Please choose the device you want to use to start the restore process." }
                            p { "You will need to compare a bunch of emojis on both devices and check that they match !" }
                        }
                    }
                }
            }

            @if !devices.is_empty() {
                form action="/tachyon/confirm_device/other_device" method="POST" ic-post-to="/tachyon/confirm_device/other_device" ic-target=".content" {
                    div id="error-message" style="display:none;" {}

                    @for device in devices {
                        input type="radio" name="device" id=(device.id.as_str()) value=(device.id.as_str()) { (device.display_name.as_deref().unwrap_or(device.id.as_str())) }
                    }

                    br {}

                    button type="submit" class="btn btn-primary" {
                        span class="btn-shine" {}
                        span class="btn-label" { "Restore this device" }
                    }
                }
            }
        }
    }
}
