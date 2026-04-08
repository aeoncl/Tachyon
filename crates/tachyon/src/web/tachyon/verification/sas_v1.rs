use matrix_sdk::encryption::verification::{SasState, SasVerification};
use matrix_sdk::ruma::UserId;
use maud::{html, Markup};

pub async fn handle_sas_v1(
    verification: SasVerification,
    refresh_url: &str,
    notification_id: i32,
    flow_id: &str,
    user_id: &UserId
) -> Markup {


    let response = match verification.state() {
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
                    div class="container" ic-src=(refresh_url) ic-replace-target="true" {
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

                           form ic-post-to="/tachyon/verification/sas_v1/confirm" ic-target="closest div.container" {
                                input type="hidden" name="notification_id" value=(notification_id);
                                input type="hidden" name="flow_id" value=(flow_id);
                                input type="hidden" name="user_id" value=(user_id);

                                button type="submit" class="btn btn-primary" {
                                    span class="btn-shine" {}
                                    span class="btn-text" { "The emojis match !" }
                                }
                            }

                            form ic-post-to="/tachyon/verification/sas_v1/mismatch" ic-target="closest div.container" {
                                input type="hidden" name="notification_id" value=(notification_id);
                                input type="hidden" name="flow_id" value=(flow_id);
                                input type="hidden" name="user_id" value=(user_id);

                                button type="submit" class="btn btn-danger" {
                                    span class="btn-shine" {}
                                    span class="btn-text" { "The emojis don't match !" }
                                }
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
        };

    response
}