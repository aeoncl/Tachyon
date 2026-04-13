use std::str::FromStr;
use axum::extract::State;
use axum::response::Html;
use maud::{html, Markup};
use crate::matrix::cross_signing::check_secret_storage_state;
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::Params;

pub(super) mod recover;
pub(super) mod reset_identity;
pub(super) mod other_device;

pub async fn get_confirm(
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

    Html(device_confirmation_content(notification_id).into_string())
}

fn device_confirmation_content(notification_id: i32) -> Markup {
    let recover_url = format!("/tachyon/confirm_device/recover?notification_id={}", notification_id);
    let other_device_url = format!("/tachyon/confirm_device/other_device?notification_id={}", notification_id);
    let reset_url = format!("/tachyon/confirm_device/reset_identity?notification_id={}", notification_id);

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
                    td class="option option-primary" valign="top" {
                        table {
                            tr {
                                td {
                                    h3 { "Use your recovery key" }
                                    p {
                                        "You can retrieve your digital identity from the server using your recovery key or passphrase."
                                    }
                                    br;
                                    a href=(recover_url) class="btn btn-primary" {
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

                    td class="option option-primary" valign="top" {
                        table {
                            tr {
                                td {
                                    h3 { "Confirm with another device" }
                                    p {
                                        "Use another confirmed device you own to exchange a copy of your digital identity."
                                    }
                                    br;
                                    a href=(other_device_url) class="btn btn-primary" {
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
                                    a href=(reset_url) class="btn btn-danger" {
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