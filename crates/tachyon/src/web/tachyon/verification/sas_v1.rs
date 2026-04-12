use axum::body::Body;
use axum::http::Response;
use axum::response::IntoResponse;
use matrix_sdk::encryption::verification::{Emoji, SasState, SasVerification};
use matrix_sdk::ruma::UserId;
use maud::{html, Markup};

pub async fn handle_sas_v1(
    verification: SasVerification,
    last_state: &str,
    refresh_url: &str,
    notification_id: i32,
    flow_id: &str,
    user_id: &UserId
) -> Response<Body> {

    let state_name = state_name(&verification.state());

    if last_state  == state_name {
        return Response::builder()
            .status(204)
            .body(Body::empty())
            .unwrap();
    }

    let refresh_url_with_state = format!("{}&state={}", refresh_url, state_name);


    let response = match verification.state() {
            SasState::Created { protocols } => {
                html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url_with_state) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "We've notified your other device" }
                                    p { "Please accept the verification invitation on your other device." }
                                }
                            }
                        }
                    }
                }
            }
            SasState::Started { protocols } => {
                verification.accept().await.unwrap();

                html! {
                    div class="container" ic-poll="1s"  ic-src=(refresh_url_with_state) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "The verification process has started" }
                                }
                            }
                        }
                    }
                }
            }
            SasState::Accepted { accepted_protocols } => {
                html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url_with_state) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Setting up device verification" }
                                    p { "Please wait while devices communicate." }
                                }
                            }
                        }
                    }
                }
            }
            SasState::KeysExchanged { emojis, decimals } => {


                let emoji_str = emojis.unwrap();
                let emojis = emoji_str.emojis;

                html! {
                    div class="container" ic-poll="2s" ic-src=(refresh_url_with_state) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Compare the emojis" }
                                    p {
                                        b{ "Check" }
                                        " if the "
                                        b{ "emojis" }
                                        " showed here "
                                        b { "match" }
                                        " with the ones on the other device."
                                        br{
                                            "If they do, "
                                            i { "you're all good." }
                                            " (H)"
                                        }
                                    }
                                }
                            }
                        }

                        (emoji_table(&emojis))

                        div class="spacer" {}

                           form class="single-btn-form" ic-post-to="/tachyon/verification/sas_v1/confirm" ic-target="closest div.container" {
                                input type="hidden" name="notification_id" value=(notification_id);
                                input type="hidden" name="flow_id" value=(flow_id);
                                input type="hidden" name="user_id" value=(user_id);

                                button type="submit" class="btn btn-primary" {
                                    span class="btn-shine" {}
                                    span class="btn-label" { "They match !" }
                                }
                            }


                            form class="single-btn-form back-btn-form" ic-post-to="/tachyon/verification/sas_v1/mismatch" ic-target="closest div.container" {
                                input type="hidden" name="notification_id" value=(notification_id);
                                input type="hidden" name="flow_id" value=(flow_id);
                                input type="hidden" name="user_id" value=(user_id);

                                button type="submit" class="btn btn-danger" {
                                    span class="btn-shine" {}
                                    span class="btn-label" { "They don't match" }
                                }
                            }

                    }
                }
            }
            SasState::Confirmed => {
                html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url_with_state) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Confirm om your other device" }
                                    p { "We are awaiting confirmation from your other device." }
                                }
                            }
                        }
                    }
                }
            }
            SasState::Done { verified_devices, verified_identities } => {
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
            SasState::Cancelled(cancel_info) => {
                html! {
                    div class="container" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Verification was cancelled" }
                                    p { "It's okay, happens to the best of us, you can always try again." }
                                }
                            }
                        }
                    }
                }
            }
        };

    response.into_response()
}

fn emoji_table(emojis: &[Emoji]) -> Markup {

    let (first_emoji_row, second_emoji_row) = emojis.split_at(4);

    html! {
        div class="emoji-container" {
            table class="emoji-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    @for emoji in first_emoji_row {
                        ( emoji_to_html(&emoji) )
                    }
                }
            }
            div class="spacer" {}
            table class="emoji-table" cellspacing="0" cellpadding="0" border="0" {
                tr {
                    @for emoji in second_emoji_row {
                        ( emoji_to_html(&emoji) )
                    }
                }
            }
        }
    }

}

fn state_name(state: &SasState) -> &'static str {
    match state {
        SasState::Created { .. } => "sas_created",
        SasState::Started { .. } => "sas_started",
        SasState::Accepted { .. } => "sas_accepted",
        SasState::KeysExchanged { .. } => "sas_keys_exchanged",
        SasState::Confirmed => "sas_confirmed",
        SasState::Done { .. } => "sas_done",
        SasState::Cancelled(_) => "sas_cancelled",
    }
}

fn emoji_to_html(emoji: &matrix_sdk::encryption::verification::Emoji) -> Markup {
    let img_url = format!("img/sas_v1/{}.gif", emoji.description.to_lowercase().replace(" ", "_"));
    html! {
        td {
            img src=(img_url) alt=(emoji.description) {}
            p { (emoji.description) }
        }
    }
}