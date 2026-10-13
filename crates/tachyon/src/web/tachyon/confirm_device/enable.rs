use axum::extract::{Extension, State};
use axum::response::{Html, IntoResponse, Response};
use maud::{html, Markup};
use log::warn;

use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::login::error_page;

pub async fn get_enable(
    State(state): State<GlobalState>,
    Extension(token): Extension<String>,
) -> Response {
    if state.get_clients(&token).is_none() {
        return error_page("No session for this token.");
    }

    Html(setup_content().into_string()).into_response()
}

pub async fn post_enable(
    State(state): State<GlobalState>,
    Extension(token): Extension<String>,
) -> Response {
    let Some(tachyon_client) = state.get_clients(&token) else {
        return error_page("No session for this token.");
    };
    let matrix_client = tachyon_client.matrix_client();

    match enable_secret_storage(&matrix_client).await {
        Ok(recovery_key) => Html(recovery_key_content(&recovery_key).into_string()).into_response(),
        Err(error) => {
            warn!("enabling secret storage failed: {}", error);
            error_page(&format!("Could not set up recovery: {}", error))
        }
    }
}

// Without an identity the device never cross-signs; this is also the only device-key upload before the first sync.
// The sync task's verification wait rechecks the cross-signed state, so no alert needs to fire here.
async fn enable_secret_storage(matrix_client: &matrix_sdk::Client) -> Result<String, anyhow::Error> {
    if !crate::matrix::cross_signing::check_device_is_crossed_signed(matrix_client).await? {
        matrix_client.encryption().bootstrap_cross_signing(None).await?;
    }

    let recovery = matrix_client.encryption().recovery();
    let recovery_key = recovery.enable().await?;
    Ok(recovery_key)
}

fn setup_content() -> Markup {
    html! {
        div class="content" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Set up recovery" }
                        p {
                            "This account has no recovery method yet. Setting one up generates your "
                            "digital identity, backs it up to the server, and gives you a recovery key."
                        }
                    }
                }
            }

            div class="sep" {}

            form method="POST" action="/tachyon/confirm_device/enable" ic-post-to="/tachyon/confirm_device/enable" ic-target=".content" {
                p { b { "You will receive a recovery key that you must store somewhere safe." } }
                div class="spacer" {}
                button type="submit" class="btn btn-primary" {
                    span class="btn-shine" {}
                    span class="btn-label" { "Set up recovery" }
                }
            }
        }
    }
}

fn recovery_key_content(recovery_key: &str) -> Markup {
    html! {
        div class="container" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Recovery is now set up" }
                        p { "Here is your recovery key, store it somewhere safe:" }
                    }
                }
            }

            div class="sep" {}

            pre style="font-size: 1.2em; text-align: center; background: #f7f7f7; border: 1px solid #ccc; padding: 10px;" { (recovery_key) }

            p style="text-align: center; color: #b30000;" {
                b { "You will not be able to see this key again." }
            }

            div class="spacer" {}

            form class="single-btn-form" method="POST" action="/tachyon/confirm_device/recovery_key/saved" ic-post-to="/tachyon/confirm_device/recovery_key/saved" ic-target=".content" {
                button type="submit" class="btn btn-primary" {
                    span class="btn-shine" {}
                    span class="btn-label" { "I saved my recovery key" }
                }
            }
        }
    }
}
