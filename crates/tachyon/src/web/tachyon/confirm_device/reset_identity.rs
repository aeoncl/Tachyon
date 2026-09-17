use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::layout::error_fragment;
use crate::web::tachyon::Params;
use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use tachyon_core::domain::auth::BridgeLinkToken;
use tachyon_core::domain::verification::{IdentityReset, Password, ResetAuth};

pub async fn get_reset_identity(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
) -> Html<String> {
    reset(&state, &token, None).await
}

pub async fn post_reset_identity(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {
    let auth = if form_data.contains_key("approved") {
        Some(ResetAuth::Approved)
    } else {
        form_data
            .get("password")
            .filter(|password| !password.is_empty())
            .map(|password| ResetAuth::Password(Password::new(password)))
    };

    reset(&state, &token, auth).await
}

/// Both entry points report the same four states, so they share one round trip and one
/// rendering. `None` asks the homeserver what it wants before anything is typed.
async fn reset(state: &GlobalState, token: &str, auth: Option<ResetAuth>) -> Html<String> {
    let use_case = state.app_state().device_verification_use_case();

    let content = match use_case.reset_identity(&BridgeLinkToken::new(token), auth).await {
        Ok(IdentityReset::PasswordRequired) => password_form(),
        Ok(IdentityReset::ApprovalRequired { url }) => approval_content(&url),
        Ok(IdentityReset::ApprovalPending) => pending_content(),
        Ok(IdentityReset::Done { recovery_key }) => done_content(recovery_key.as_str()),
        Err(e) => error_fragment(e),
    };

    Html(content.into_string())
}

fn password_form() -> Markup {
    html! {
        div class="container" {
            h2 class="h3-danger" { "Reset your digital identity" }
            p { "Your homeserver needs your account password to replace your identity." }
            form action="/tachyon/confirm_device/reset_identity" method="POST" ic-post-to="/tachyon/confirm_device/reset_identity" ic-target="closest div.container" {
                div id="error-message" style="display:none;" {}
                input type="password" name="password" id="password" {}
                button type="submit" class="btn btn-danger" {
                    span class="btn-shine" {}
                    span class="btn-label" { "Reset Identity" }
                }
            }
        }
    }
}

fn approval_content(url: &str) -> Markup {
    html! {
        div class="container" {
            h2 class="h3-danger" { "Approve the reset" }
            p { "Your homeserver wants you to approve this reset in your browser first." }
            p { a href=(url) target="_blank" { "Open the approval page" } }
            form action="/tachyon/confirm_device/reset_identity" method="POST" ic-post-to="/tachyon/confirm_device/reset_identity" ic-target="closest div.container" {
                input type="hidden" name="approved" value="yes";
                button type="submit" class="btn btn-danger" {
                    span class="btn-shine" {}
                    span class="btn-label" { "I've approved, continue" }
                }
            }
        }
    }
}

fn pending_content() -> Markup {
    html! {
        div class="container" ic-poll="2s" ic-src="/tachyon/confirm_device/reset_identity" ic-replace-target="true" {
            h2 { "Resetting your identity" }
            p { "This is running against your homeserver, please wait." }
        }
    }
}

fn done_content(recovery_key: &str) -> Markup {
    html! {
        div class="container" {
            h2 { "Your device is now confirmed!" }
            p { "Here is your new recovery key. Store it somewhere safe, it is shown only once:" }
            pre { (recovery_key) }
            p { "You can go back to Messenger now." }
        }
    }
}
