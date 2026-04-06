use std::str::FromStr;
use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use crate::matrix::cross_signing::restore_from_recovery_key;
use crate::tachyon::alert::{AlertError, AlertNotify, AlertSuccess};
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::Params;

pub async fn get_restore(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Html<String> {
    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let _notification = tachyon_client.alerts().get(&notification_id).unwrap();
    let _matrix_client = state.matrix_clients().get(&token).unwrap();

    Html(restore_device_content(notification_id).into_string())
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
                        span class="btn-label" { "Restore this device" }
                    }

                }
            }
        }
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
