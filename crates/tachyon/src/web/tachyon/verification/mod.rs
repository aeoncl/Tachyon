pub(super) mod sas_v1_actions;
pub(super) mod sas_v1;

use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::Params;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use matrix_sdk::encryption::verification::VerificationRequestState;
use matrix_sdk::ruma::OwnedUserId;
use maud::{html, Markup};
use std::str::FromStr;
use anyhow::anyhow;
use axum::http::{Response, StatusCode};
use axum::http::header::CONTENT_TYPE;
use matrix_sdk::ruma::events::key::verification::VerificationMethod;
use mime::TEXT_HTML;
use crate::tachyon::alert::{AlertNotify, AlertSuccess};

pub async fn get_verification_poll(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> impl IntoResponse {
    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let flow_id = params.get("flow_id").map(|s| s.as_str()).unwrap_or_default();

    let user_id_raw = params.get("user_id").map(|s| s.as_str()).unwrap_or_default();
    let user_id = OwnedUserId::from_str(user_id_raw).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    if !tachyon_client.alerts().contains_key(&notification_id)
    {
        panic!("Notification not found");
    }

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
            let sas_v1 = their_methods.iter().any(|method| matches!(method, VerificationMethod::SasV1));
            if !sas_v1 {
                verification_request.cancel().await.unwrap();
            } else {
                verification_request.accept_with_methods(vec![VerificationMethod::SasV1]).await.unwrap();
            }

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
            if let Some(verification) = verification.sas() {
                sas_v1::handle_sas_v1(verification, &refresh_url, notification_id, flow_id, &user_id).await
            } else {
                html! {
                    div class="container" {
                        table class="hero-table" cellspacing="0" cellpadding="0" border="0" {
                            tr {
                                td class="hero-text" valign="middle" {
                                    h2 { "Unsupported verification method" }
                                }
                            }
                        }
                    }
                }
            }

        }
        VerificationRequestState::Done => {

            let (_, notification) = tachyon_client.alerts().remove(&notification_id).unwrap();
            notification.notify_success(AlertSuccess::Unit);

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

            let (_, notification) = tachyon_client.alerts().remove(&notification_id).unwrap();
            notification.notify_failure(anyhow!("Verification cancelled: {:?}", cancelled));

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

    response
}

