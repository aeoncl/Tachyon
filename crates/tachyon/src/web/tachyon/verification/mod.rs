pub(super) mod sas_v1_actions;

use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::layout::error_html;
use crate::web::tachyon::Params;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode};
use axum::response::{Html, IntoResponse};
use maud::{html, Markup};
use tachyon_core::domain::auth::BridgeLinkToken;
use tachyon_core::domain::verification::{SasEmoji, VerificationFlowState};

pub async fn get_verification_poll(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Response<Body> {
    let use_case = state.app_state().device_verification_use_case();

    let flow_state = match use_case.verification_state(&BridgeLinkToken::new(&token)) {
        Ok(flow_state) => flow_state,
        Err(e) => return error_html(e).into_response(),
    };

    let shown = params.get("state").map(|state| state.trim()).unwrap_or_default();
    if shown == flow_state.name() {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap();
    }

    let poll_url = format!("/tachyon/verification?state={}", flow_state.name());

    let content = match flow_state {
        VerificationFlowState::Requested => waiting_content(
            &poll_url,
            "Verification request sent !",
            "Awaiting confirmation from the other side.",
        ),
        VerificationFlowState::Ready => waiting_content(
            &poll_url,
            "Your other device accepted",
            "Waiting for the emoji exchange to start.",
        ),
        VerificationFlowState::Started => waiting_content(
            &poll_url,
            "Setting up device verification",
            "Please wait while devices communicate.",
        ),
        VerificationFlowState::CompareEmojis { emojis } => {
            compare_emojis_content(&poll_url, &emojis)
        }
        VerificationFlowState::AwaitingOtherConfirmation => waiting_content(
            &poll_url,
            "Confirm on your other device",
            "We are awaiting confirmation from your other device.",
        ),
        VerificationFlowState::Done => done_content(),
        VerificationFlowState::Cancelled { .. } => cancelled_content(),
    };

    Html(content.into_string()).into_response()
}

fn waiting_content(poll_url: &str, title: &str, detail: &str) -> Markup {
    html! {
        div class="container" ic-poll="1s" ic-src=(poll_url) ic-replace-target="true" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-text" valign="middle" {
                        h2 { (title) }
                        p { (detail) }
                    }
                }
            }
        }
    }
}

fn compare_emojis_content(poll_url: &str, emojis: &[SasEmoji]) -> Markup {
    html! {
        div class="container" ic-poll="2s" ic-src=(poll_url) ic-replace-target="true" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Compare the emojis" }
                        p {
                            b { "Check" }
                            " if the "
                            b { "emojis" }
                            " showed here "
                            b { "match" }
                            " with the ones on the other device."
                            br {
                                "If they do, "
                                i { "you're all good." }
                                " (H)"
                            }
                        }
                    }
                }
            }

            (emoji_table(emojis))

            div class="spacer" {}

            form class="single-btn-form" ic-post-to="/tachyon/verification/sas_v1/confirm" ic-target="closest div.container" {
                button type="submit" class="btn btn-primary" {
                    span class="btn-shine" {}
                    span class="btn-label" { "They match !" }
                }
            }

            form class="single-btn-form back-btn-form" ic-post-to="/tachyon/verification/sas_v1/mismatch" ic-target="closest div.container" {
                button type="submit" class="btn btn-danger" {
                    span class="btn-shine" {}
                    span class="btn-label" { "They don't match" }
                }
            }
        }
    }
}

fn done_content() -> Markup {
    html! {
        div class="container" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Your device is now verified !!" }
                        p { "Congraaaattzzz" }
                    }
                }
            }
        }
    }
}

fn cancelled_content() -> Markup {
    html! {
        div class="container" {
            table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    td class="hero-text" valign="middle" {
                        h2 { "Verification was cancelled" }
                        p { "It's okay, happens to the best of us, you can always try again. (yn)" }
                    }
                }
            }
        }
    }
}

fn emoji_table(emojis: &[SasEmoji]) -> Markup {
    html! {
        div class="emoji-container" {
            @for row in emojis.chunks(4) {
                table class="emoji-table" cellspacing="0" cellpadding="0" border="0" {
                    tr {
                        @for emoji in row {
                            (emoji_cell(emoji))
                        }
                    }
                }
                div class="spacer" {}
            }
        }
    }
}

fn emoji_cell(emoji: &SasEmoji) -> Markup {
    let img_url = format!(
        "img/sas_v1/{}.gif",
        emoji.description.to_lowercase().replace(' ', "_")
    );
    html! {
        td {
            img src=(img_url) alt=(emoji.description) {}
            p { (emoji.description) }
        }
    }
}
