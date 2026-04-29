use crate::matrix::login::login_with_password;
use crate::tachyon::global_state::GlobalState;
use crate::web::soap::error::RST2Error;
use crate::web::tachyon::{layout, Params};
use anyhow::anyhow;
use axum::extract::State;
use axum::response::Html;
use maud::html;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::str::FromStr;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;

pub async fn get_auth(
    axum::extract::Query(params): axum::extract::Query<Params>
) -> Html<String> {

    let username = params.get("username").map(|s| s.as_str()).unwrap_or_default();

    let content = html! {
        form method="POST" ic-post-to="/tachyon/auth" ic-target=".content" {
            input type="text" name="username" placeholder="Username" value=(username);
            input type="password" name="password" placeholder="Password";
            button type="submit" { "Log In" }
        }
    };

    Html(layout::tachyon_page_no_nav(content).into_string())
}

pub async fn post_auth(
    State(state): State<GlobalState>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {

    let username = form_data.get("username").map(|s| s.as_str()).unwrap_or_default();
    let password = form_data.get("password").map(|s| s.as_str()).unwrap_or_default();

    let email = EmailAddress::from_str(username).unwrap();
    let matrix_id = email.to_owned_user_id();

    let login_successful =
        if let Ok((matrix_token, _)) = login_with_password(matrix_id, password, !state.get_config().strict_ssl).await {
            let ticket_token = TicketToken(
                state
                    .secret_encryptor()
                    .encrypt(&matrix_token)
                    .map_err(|e| {
                        RST2Error::InternalServerError {
                            source: anyhow!("Failed to encrypt token: {}", e),
                        }
                    })
                    .unwrap(),
            );
            state.store_pending_ticket(email, ticket_token);
            true
        } else {
            false
        };

    let page = html! {
        div class="container" {
            div class="signin" {
                h2 { "Log-in Result" }
                @if login_successful {
                    p { "Login successful for " (username) "!" }
                } @else {
                    p { "Login failed. Please try again." }
                }
            }
        }
    };

    Html(page.into_string())
}