use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::layout::error_fragment;
use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use tachyon_core::application::device_verification_use_case::DeviceVerificationUseCase;
use tachyon_core::application::error::VerificationError;
use tachyon_core::domain::auth::BridgeLinkToken;
use tachyon_core::domain::verification::{DeviceStatus, VerificationOptions};

pub(super) mod recover;
pub(super) mod reset_identity;
pub(super) mod other_device;

pub async fn get_confirm(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
) -> Html<String> {
    let use_case = state.app_state().device_verification_use_case();

    let content = match confirm_content(use_case, &BridgeLinkToken::new(&token)).await {
        Ok(content) => content,
        Err(e) => error_fragment(e),
    };

    Html(content.into_string())
}

async fn confirm_content(
    use_case: &DeviceVerificationUseCase,
    token: &BridgeLinkToken,
) -> Result<Markup, VerificationError> {
    Ok(match use_case.status(token).await? {
        DeviceStatus::Verified => already_confirmed_content(),
        DeviceStatus::Unverified => device_confirmation_content(&use_case.options(token).await?),
    })
}

fn already_confirmed_content() -> Markup {
    html! {
        div class="container" {
            h2 { "This device is confirmed" }
            p { "You can go back to Messenger now." }
        }
    }
}

fn device_confirmation_content(options: &VerificationOptions) -> Markup {
    html! {
        div class="content" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Confirm it's you" }
                        p {
                            "This device is not confirmed yet. This step allows your contacts to trust that "
                            i { "you are you™" }
                            br;
                            "Please choose one of the following options:"
                        }
                    }
                }
            }

            div class="sep" {}

            table class="options" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    @if options.recovery_available {
                        td class="option option-primary" valign="top" {
                            table {
                                tr {
                                    td {
                                        h3 { "Use your recovery key" }
                                        p {
                                            "You can retrieve your digital identity from the server using your recovery key or passphrase."
                                        }
                                        br;
                                        a href="/tachyon/confirm_device/recover" class="btn btn-primary" {
                                            span class="btn-shine" {}
                                            span class="btn-label" { "Confirm with recovery" }
                                        }
                                    }
                                    td class="hero-icon" {
                                        img src="img/text-looking.gif" alt="Smiley looking at text";
                                    }
                                }
                            }
                        }
                    }

                    @if !options.devices.is_empty() {
                        td class="option option-primary" valign="top" {
                            table {
                                tr {
                                    td {
                                        h3 { "Confirm with another device" }
                                        p {
                                            "Use another confirmed device you own to exchange a copy of your digital identity."
                                        }
                                        br;
                                        a href="/tachyon/confirm_device/other_device" class="btn btn-primary" {
                                            span class="btn-shine" {}
                                            span class="btn-label" { "Confirm with device" }
                                        }
                                    }
                                    td class="hero-icon" {
                                        img src="img/smiley_bosseordi.gif" alt="Smiley computer";
                                    }
                                }
                            }
                        }
                    }
                }
            }

            div class="sep sep-secondary" {}

            table class="options" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="option option-secondary" valign="top" {
                        table {
                            tr {
                                td class="hero-icon" {
                                    img src="img/scared-emoticon.gif" alt="Scared";
                                }
                                td {
                                    h3 class="h3-danger" { "Reset your digital identity" }
                                    p {
                                        "Last resort option if you have forgotten your recovery key and have lost access to all your confirmed devices. You will lose your encrypted chat history :c. This operation is irreversible."
                                    }
                                    br;
                                    a href="/tachyon/confirm_device/reset_identity" class="btn btn-danger" {
                                        span class="btn-shine" {}
                                        span class="btn-label" { "Reset identity" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
