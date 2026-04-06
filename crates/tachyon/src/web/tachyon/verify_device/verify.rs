use std::str::FromStr;
use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use crate::matrix::cross_signing::check_secret_storage_state;
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

    let secret_store_enabled = check_secret_storage_state(&matrix_client).await.unwrap();

    Html(device_verification_page(notification_id, secret_store_enabled).into_string())
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
                            p { "It looks like you don't have a recovery key set up. You can either set one up using another device (Recommended) or reset your account and create a new recovery key." }
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
                                h3 { "Restore this device" }
                            }
                            p {
                                "Use your existing recovery key or passphrase to verify this device and restore access to all your messages and backups. "
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
                            "Start fresh by resetting your account. All previously verified sessions will be invalidated. You will lose access to your encrypted message history."
                            br;
                            " This action cannot be undone."
                        }
                        a href=(reset_url) ic-get-from=(reset_url) ic-target=".content" ic-push-url="true" class="btn btn-danger" {
                            span class="btn-shine" {}
                            span class="btn-label" { "Reset account" }
                        }
                    }
                }
            }
        }
    }
}