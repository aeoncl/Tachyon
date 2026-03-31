use std::str::FromStr;
use anyhow::anyhow;
use crate::tachyon::tachyon_state::TachyonState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Response, StatusCode};
use axum::response::Html;
use axum::routing::{get, post};
use axum::Router;
use lazy_static_include::lazy_static_include_bytes;
use maud::{html, Markup, DOCTYPE};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use crate::matrix::login::login_with_password;
use crate::tachyon::identifiers::MatrixIdCompatible;
use crate::web::soap::error::RST2Error;

lazy_static_include_bytes! {
    INDEX => "./assets/web/tachyon/index.html",
    FAVICON => "./assets/web/tachyon/favicon.ico",
    STYLE => "./assets/web/tachyon/style.css",
    LOGO => "./assets/web/tachyon/tachyon_logo.png",
    LOGO_2 => "./assets/web/tachyon/tachyon_logo_2.png",
    INTERCOOLER => "./assets/web/tachyon/intercooler-1.2.4.min.js",
    JQUERY => "./assets/web/tachyon/jquery-1.10.0.min.js"
}
pub fn tachyon_router(state: TachyonState) -> Router<TachyonState> {
    Router::new()
        .route("/", get(serve_index))
        .route("/auth", get(get_auth))
        .route("/auth", post(post_auth))
        .route("/{file}", get(serve_static))
        .with_state(state)
}

pub fn index_page() -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                title { "Tachyon" }
                base href="/tachyon/";
                link rel="icon" type="image/x-icon" href="favicon.ico";
                link rel="stylesheet" href="style.css";
                script type="text/javascript" src="jquery-1.10.0.min.js" {}
                script type="text/javascript" src="intercooler-1.2.4.min.js" {}
            }
            body {
                div class="header" {
                    div class="bg" {
                        div class="bg-content" {
                            img class="logo" src="tachyon_logo_2.png" alt="Tachyon Logo";
                            div class="title" {
                                h1 { "Tachyon" }
                                h2 { "Welcome to Tachyon" }
                            }

                            div class="menu" {
                                ul {
                                    li { "Home" }
                                    li { "Profile" }
                                    li { "People" }
                                }
                            }

                            div class="signin" {
                                h2 { "Log-on" }
                            }
                        }
                    }
                }

                div class="content" {}
            }
        }
    }
}

async fn serve_index() -> Html<String> {
    Html(index_page().into_string())
}

async fn get_auth(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> Html<String> {

    let username = params.get("username").map(|s| s.as_str()).unwrap_or("");

    let page =  html! {
        (DOCTYPE)
        html {
            head {
                title { "Tachyon" }
                base href="/tachyon/";
                link rel="icon" type="image/x-icon" href="favicon.ico";
                link rel="stylesheet" href="style.css";
                script type="text/javascript" src="jquery-1.10.0.min.js" {}
                script type="text/javascript" src="intercooler-1.2.4.min.js" {}
            }
            body {
                div class="header" {
                    div class="bg" {
                        div class="bg-content" {
                            img class="logo" src="tachyon_logo_2.png" alt="Tachyon Logo";
                            div class="title" {
                                h1 { "Tachyon" }
                                h2 { "Welcome to Tachyon" }
                            }

                            div class="menu" {
                                ul {
                                    li { "Home" }
                                    li { "Profile" }
                                    li { "People" }
                                }
                            }

                            div class="signin" {
                                h2 { "Log-on" }
                            }
                        }
                    }
                }

                div class="content" {
                    form method="POST" ic-post-to="/tachyon/auth" ic-target=".content" {
                        input type="text" name="username" placeholder="Username" value=(username);
                        input type="password" name="password" placeholder="Password";
                        button type="submit" { "Log In" }
                    }
                }
            }
        }
    };

    Html(page.into_string())
}

async fn post_auth(
    State(state): State<TachyonState>,
    axum::extract::Form(form_data): axum::extract::Form<std::collections::HashMap<String, String>>
) -> Html<String> {

    let username = form_data.get("username").map(|s| s.as_str()).unwrap_or("Unknown");
    let password = form_data.get("password").map(|s| s.as_str()).unwrap_or("");

    let email = EmailAddress::from_str(&username).unwrap();

    let matrix_id = email.to_owned_user_id();

    let login_successful = if let Ok((matrix_token, _client)) = login_with_password(matrix_id, &password, true).await {
        let ticket_token = TicketToken(state.secret_encryptor().encrypt(&matrix_token)
            .map_err(|e| RST2Error::InternalServerError { source: anyhow!("Failed to encrypt token: {}", e) }).unwrap()
        );

        state.store_pending_ticket(email.to_string(), ticket_token);

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

async fn serve_static(Path(file): Path<String>) -> Response<Body> {
    let (data, content_type) = match file.as_str() {
        "favicon.ico" => (*FAVICON, "image/x-icon"),
        "style.css" => (*STYLE, "text/css"),
        "tachyon_logo.png" => (*LOGO, "image/png"),
        "tachyon_logo_2.png" => (*LOGO_2, "image/png"),
        "intercooler-1.2.4.min.js" => (*INTERCOOLER, "text/javascript"),
        "jquery-1.10.0.min.js" => (*JQUERY, "text/javascript"),
        _ => return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    };

    Response::builder()
        .header(CONTENT_TYPE, content_type)
        .body(Body::from(data))
        .expect("response to be valid")
}