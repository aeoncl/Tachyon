use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::layout::error_fragment;
use crate::web::tachyon::Params;
use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use tachyon_core::application::error::VerificationError;
use tachyon_core::domain::auth::BridgeLinkToken;
use tachyon_core::domain::verification::RecoveryKey;

pub async fn get_recover() -> Html<String> {
    Html(restore_device_content(None).into_string())
}

pub async fn post_recover(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {
    let secret = match form_data.get("restore_method").map(String::as_str) {
        Some("passphrase") => form_data.get("passphrase"),
        _ => form_data.get("recovery_key_full"),
    };

    let Some(secret) = secret.map(|s| s.trim()).filter(|s| !s.is_empty()) else {
        return Html(
            restore_device_content(Some("Please fill in your recovery key or passphrase."))
                .into_string(),
        );
    };

    let use_case = state.app_state().device_verification_use_case();
    let content = match use_case
        .recover(&BridgeLinkToken::new(&token), &RecoveryKey::new(secret))
        .await
    {
        Ok(()) => confirmed_content(),
        Err(VerificationError::RecoveryKeyRejected) => restore_device_content(Some(
            "That recovery key or passphrase was not accepted. Please check it and try again.",
        )),
        Err(e) => error_fragment(&e.to_string()),
    };

    Html(content.into_string())
}

fn confirmed_content() -> Markup {
    html! {
        div class="container" {
            h2 { "Your device is now confirmed!" }
            p { "You can go back to Messenger now." }
        }
    }
}

fn restore_device_content(error: Option<&str>) -> Markup {
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

            form action="/tachyon/confirm_device/recover" method="POST" ic-post-to="/tachyon/confirm_device/recover" ic-target=".content" ic-on-beforeSend="if (!validateForm()) { settings.cancel = true; return false;}" {
                @match error {
                    Some(error) => { div id="error-message" { (error) } }
                    None => { div id="error-message" style="display:none;" {} }
                }

                div class="restore-options-container" {
                    div class="option option-primary clickable-option" id="card-recovery-key" {
                        input type="radio" name="restore_method" id="use-recovery-key" value="recovery-key" checked;
                        table class="option-content" {
                            tr {
                                td {
                                    img src="img/key.gif" alt="Recovery Key" class="option-icon";
                                }
                                td {
                                    h3 {
                                        label for="use-recovery-key" { "Recovery Key" }
                                    }
                                    p { "Use the 48-character recovery key that was generated during recovery setup." }
                                }
                            }
                        }
                    }

                    div class="option option-primary clickable-option" id="card-passphrase" {
                        input type="radio" name="restore_method" id="use-passphrase" value="passphrase";
                        table class="option-content" {
                            tr {
                                td {
                                    img src="img/star_speech.gif" alt="Passphrase" class="option-icon";
                                }
                                td {
                                    h3 {
                                        label for="use-passphrase" { "Passphrase" }
                                    }
                                    p { "Use the security passphrase that you chose during recovery setup." }
                                }
                            }
                        }
                    }
                }

                div class="spacer" {}
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
