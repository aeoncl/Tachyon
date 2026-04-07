use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::Params;
use axum::extract::State;
use axum::response::Html;
use matrix_sdk::encryption::verification::{SasState, Verification, VerificationRequestState};
use matrix_sdk::ruma::OwnedUserId;
use maud::{html, Markup};
use std::str::FromStr;

pub async fn get_verification(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Html<String> {
    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let flow_id = params.get("flow_id").map(|s| s.as_str()).unwrap_or_default();

    let user_id_raw = params.get("user_id").map(|s| s.as_str()).unwrap_or_default();
    let user_id = OwnedUserId::from_str(user_id_raw).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let _notification = tachyon_client.alerts().get(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    let verification_request = state.pending_verification_requests().get(&flow_id).unwrap();

    let refresh_url = format!("/tachyon/verification?notification_id={}&flow_id={}&user_id={}", notification_id, flow_id, user_id_raw);

    let response = match verification_request.state() {
        VerificationRequestState::Created { our_methods } => {
            html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "We have sent a verification request to your other device" }
                                    p { "Please accept the verification invitation on your other device." }
                                }
                            }
                        }
                    }

                }
        }
        VerificationRequestState::Requested { their_methods, other_device_data } => {
            html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "We received an invitation from another device" }
                                }
                            }
                        }
                    }
                }
        }
        VerificationRequestState::Ready { their_methods, our_methods, other_device_data } => {
            verification_request.accept().await.unwrap();

            html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "We are ready to perform the verification" }
                                }
                            }
                        }
                    }
                }

        }
        VerificationRequestState::Transitioned { verification } => {

            handle_verification(verification, &refresh_url).await


        }
        VerificationRequestState::Done => {
            html! {
                    div class="container" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Verification completed successfully (request)" }
                                }
                            }
                        }
                    }
                }
        }
        VerificationRequestState::Cancelled(cancelled) => {
            html! {
                    div class="container" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Verification was cancelled (request)" }
                                    p { "It's okay, happens to the best of us, you can always try again." }
                                }
                            }
                        }
                    }
                }
        }
    };







    Html(response.into_string())
}

async fn handle_verification(verification: Verification, refresh_url: &str) -> Markup {
    let response = if let Some(verification) = verification.sas() {

        match verification.state() {
            SasState::Created { protocols } => {

                html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url) ic-replace-target="true" {
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
                    div class="container" ic-poll="1s"  ic-src=(refresh_url) ic-replace-target="true" {
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
                    div class="container" ic-poll="1s" ic-src=(refresh_url) ic-replace-target="true" {
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

                let emoji_array = emoji_str.emojis;

                html! {
                    div class="container" ic-poll="1s"  ic-src=(refresh_url) ic-replace-target="true" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Compare these emojis on both your devices" }
                                    p { "Please wait while devices communicate." }
                                }
                            }
                        }


                        @for emoji in emoji_array {
                            span { (emoji.symbol) }
                        }

                    }
                }
            }
            SasState::Confirmed => {
                html! {
                    div class="container" ic-poll="1s" ic-src=(refresh_url) ic-replace-target="true" {
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
        }
    } else {
        html! {
            div class="container" {
                table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                    tr {
                        td class="hero-text" valign="middle" {
                            h2 { "Verification failed" }
                            p { "The verification process failed." }
                        }
                    }
                }
            }
        }
    };

    response
}