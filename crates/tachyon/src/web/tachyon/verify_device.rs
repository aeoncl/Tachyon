use std::fmt::format;
use std::iter::Map;
use std::str::FromStr;
use axum::extract::{Multipart, State};
use axum::http::HeaderMap;
use axum::response::Html;
use matrix_sdk::ruma::api::client::uiaa::{AuthData, Password, UserIdentifier};
use maud::{html, Markup, PreEscaped};
use crate::matrix::cross_signing::{check_secret_storage_state, restore_from_recovery_key};
use crate::tachyon::alert::{AlertError, AlertNotify, AlertSuccess};
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::{layout, Params, VERIFY_SCRIPT};

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

    let secret_store_enabled = check_secret_storage_state(&matrix_client).await.unwrap();

    Html(layout::tachyon_page(device_verification_page(notification_id, secret_store_enabled)).into_string())
}

fn device_verification_page(notification_id: i32, secret_store_enabled: bool) -> Markup {
    let restore_url = format!("/tachyon/verify_device/restore?notification_id={}", notification_id);
    let reset_url = format!("/tachyon/verify_device/reset_identity?notification_id={}", notification_id);

    let reset_option_class = if secret_store_enabled { "option-secondary" } else { "option-primary" };

    html! {
        div class="content" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-icon" valign="middle" {
                        img src="shield_verify.png" width="48" height="48" alt="";
                    }
                    td class="hero-text" valign="middle" {
                        h2 { "Device Verification Required" }
                        @if secret_store_enabled {
                            p { "This device has not been verified yet. You can use your recovery key to restore access." }
                        } @else {
                            p { "It looks like you don't have a recovery key set up. You can reset your cryptographic identity to get a recovery key." }
                        }
                    }
                }
            }

            div class="sep" {}

            table class="options" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    @if secret_store_enabled {
                        td class="option option-primary" valign="top" {
                            a href=(restore_url) ic-get-from=(restore_url) ic-push-url="true" ic-target=".content" {
                                h3 { "Restore with recovery key" }
                            }
                            p {
                                "Use your existing recovery key to verify this device and restore access to all your messages and backups. "
                                br;
                                " This is the recommended option."
                            }
                            a href=(restore_url) ic-get-from=(restore_url) ic-target=".content" ic-push-url="true" class="btn btn-primary" {
                                span class="btn-shine" {}
                                span class="btn-label" { "Restore this device" }
                            }
                        }
                    }

                    td class={"option " (reset_option_class)} valign="top" {
                        h3 { "Reset your account" }
                        p {
                            "Start fresh by resetting your account's backup. All previously verified sessions will be invalidated. You will lose access to your encrypted message history. "
                            br;
                            " This action cannot be undone."
                        }
                        a href=(reset_url) ic-get-from=(reset_url) ic-target=".content" ic-push-url="true" class="btn btn-danger" {
                            span class="btn-shine" {}
                            span class="btn-label" { "Reset identity" }
                        }
                    }
                }
            }
        }
    }
}


pub async fn get_restore(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
    headers: HeaderMap,
) -> Html<String> {

    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let _notification = tachyon_client.alerts().get(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    let content = restore_device_content(notification_id);

    //TODO move this in a middleware and return fragments everywhere.
    let is_ic_request = headers.get("X-IC-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v == "true")
        .unwrap_or(false);

    if is_ic_request {
        Html(content.into_string())
    } else {
        Html(layout::tachyon_page(content).into_string())
    }
}

fn restore_device_content(notification_id: i32) -> Markup {

    html! {
        div class="content" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
            tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Restore this device" }
                        p { "Please fill in your recovery key or passphrase" }
                    }
                }
            }

            form action="/tachyon/verify_device/restore" method="POST" ic-post-to="/tachyon/verify_device/restore" ic-target=".content" ic-on-beforeSend="if (!validateForm()) { settings.cancel = true; return false;}" {
                div id="error-message" style="display:none;" {}

                table class="restore-options" cellspacing="0" cellpadding="0" {
                    tr {
                        td class="option option-primary clickable-option" id="card-recovery-key" {
                            input type="radio" name="restore_method" id="use-recovery-key" value="recovery-key" checked;
                            h3 {
                                label for="use-recovery-key" { "Recovery Key" }
                            }
                            p { "Use your 48-character recovery key to restore access to your encrypted messages." }
                        }
                        td class="option option-secondary clickable-option" id="card-passphrase" {
                            input type="radio" name="restore_method" id="use-passphrase" value="passphrase";
                            h3 {
                                label for="use-passphrase" { "Passphrase" }
                            }
                            p { "Use your security passphrase if you set one up during recovery key creation." }
                        }
                    }
                }

                div class="sep" {}

                div id="recovery-key-section" {
                    label { "Enter your Recovery Key" }

                    table class="cd-key-container" cellspacing="0" cellpadding="0" {
                        tr {
                            @for i in 0..6 {
                                td { input type="text" class="cd-key-block" maxlength="4" data-index=(i); }
                                @if i < 5 {
                                    td class="cd-key-separator" { "-" }
                                }
                            }
                        }
                        tr {
                            @for i in 6..12 {
                                td { input type="text" class="cd-key-block" maxlength="4" data-index=(i); }
                                @if i < 11 {
                                    td class="cd-key-separator" { "-" }
                                    }
                                }
                            }
                        }
                    }
                    input type="hidden" id="recovery_key_full" name="recovery_key_full";
                    input type="hidden" id="notification_id" name="notification_id" value=(notification_id);
                    div id="passphrase-section" style="display:none;" {
                        label for="passphrase" { "Enter your Passphrase" }
                        input type="password" id="passphrase" name="passphrase" style="width: 300px;";
                    }

                    button type="submit" class="btn btn-primary" {
                        span class="btn-shine" {}
                        span class="btn-label" { "Restore device" }
                    }

                }
            }
        }
}


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
    let matrix_client = state.matrix_clients().get(&token).unwrap();

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
            p { "Here is your recovery key, store it somewhere safe:" }
            pre { (secret_storage_key.to_string())}
        } @else {
            p { "Your device could not be verified :(" }
        }
    };

    Html(page.into_string())
}

pub async fn post_restore(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {


    let notification_id_raw = form_data.get("notification_id").map(|s| s.as_str()).unwrap();
    let notification_id = i32::from_str(notification_id_raw).unwrap();

    let passphrase = form_data.get("passphrase").map(|s| s.as_str()).unwrap();
    let recovery_key_full = form_data.get("recovery_key_full").map(|s| s.as_str()).unwrap();
    let is_recovery_key = form_data.get("restore_method").map(|s| s.as_str()).unwrap() == "recovery-key";

    let recovery_key = if is_recovery_key {
        recovery_key_full
    } else {
        passphrase
    };


    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let (_id, mut alert) = tachyon_client.alerts().remove(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    println!("Recovering device with recovery key: {}", recovery_key);

    let result = restore_from_recovery_key(&matrix_client, recovery_key).await;

    let successful = result.is_ok();

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