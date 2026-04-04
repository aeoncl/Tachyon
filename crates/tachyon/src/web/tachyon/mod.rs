use crate::matrix::login::login_with_password;
use crate::tachyon::identifiers::MatrixIdCompatible;
use crate::tachyon::tachyon_client::{Alert, AlertType};
use crate::tachyon::tachyon_state::{Repository, TachyonState};
use crate::web::soap::error::RST2Error;
use anyhow::anyhow;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE};
use axum::http::{Request, Response, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use lazy_static_include::lazy_static_include_bytes;
use matrix_sdk::ruma::exports::http::header::SET_COOKIE;
use maud::{html, Markup, PreEscaped, DOCTYPE};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::{NotServer, NotificationPayloadType};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::str::FromStr;

lazy_static_include_bytes! {
    INDEX => "./assets/web/tachyon/index.html",
    FAVICON => "./assets/web/tachyon/favicon.ico",
    STYLE => "./assets/web/tachyon/style.css",
    LOGO => "./assets/web/tachyon/tachyon_logo.png",
    LOGO_2 => "./assets/web/tachyon/tachyon_logo_2.png",
    INTERCOOLER => "./assets/web/tachyon/intercooler-1.2.4.min.js",
    JQUERY => "./assets/web/tachyon/jquery-1.10.0.min.js"
}

// ──────────────────────────────────────────────
// Shared layout
// ──────────────────────────────────────────────

fn page_head() -> Markup {
    html! {
        head {
            title { "Tachyon" }
            base href="/tachyon/";
            link rel="icon" type="image/x-icon" href="favicon.ico";
            link rel="stylesheet" href="style.css";
            script type="text/javascript" src="jquery-1.10.0.min.js" {}
            script type="text/javascript" src="intercooler-1.2.4.min.js" {}
        }
    }
}

fn page_header_bg(show_nav: bool) -> Markup {
    html! {
        div class="header" {
            div class="bg" {
                div class="bg-content" {
                    img class="logo" src="tachyon_logo_2.png" alt="Tachyon Logo";
                    div class="title" {
                        h1 { "Tachyon" }
                        h2 { "Welcome to Tachyon" }
                    }

                    @if show_nav {
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
        }
    }
}

fn layout(content: Markup, show_nav: bool) -> Markup {
    html! {
        (DOCTYPE)
        html {
            (page_head())
            body {
                (page_header_bg(show_nav))
                div class="content" {
                    (content)
                }
            }
        }
    }
}

pub fn index_page(content: Markup) -> Markup {
    layout(content, true)
}

fn error_page(message: String) -> Markup {
    layout(
        html! {
            h2 { "Something went wrong" }
            p { (message) }
        },
        false,
    )
}

// ──────────────────────────────────────────────
// Helpers
// ──────────────────────────────────────────────

type Params = std::collections::HashMap<String, String>;

fn get_param<'a>(params: &'a Params, key: &str) -> Option<&'a str> {
    params.get(key).map(|s| s.as_str())
}

fn require_param<'a>(params: &'a Params, key: &str) -> Result<&'a str, String> {
    get_param(params, key).ok_or_else(|| format!("Missing required parameter: {}", key))
}

fn parse_notification_id(params: &Params) -> Result<i32, String> {
    let raw = require_param(params, "notification_id")?;
    i32::from_str(raw).map_err(|e| format!("Invalid notification_id: {}", e))
}

fn parse_email(raw: &str) -> Result<EmailAddress, String> {
    EmailAddress::from_str(raw).map_err(|e| format!("Invalid email address: {}", e))
}

fn webauth_poll_snippet(notification_id: i32, email: &str) -> Markup {
    html! {
        div class="container" ic-poll="2s" ic-src={"/tachyon/webauth?notification_id=" (notification_id) "&email=" (email)} {
            h2 { "Tachyon Web Login" }
            p { "A notification has been sent to your client. Please approve the login." }
        }
    }
}

fn login_form() -> Markup {
    html! {
        h2 { "Tachyon Web Login" }
        p { "If your client is logged in, fill in your email address and you will receive a notification." }
        form method="POST" ic-post-to="/tachyon/webauth" ic-target=".content" {
            input type="text" name="email" placeholder="Email";
            button type="submit" { "Request Login" }
        }
    }
}

fn clear_token_cookie() -> &'static str {
    "t=; Max-Age=-1; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax;"
}

fn set_token_cookie(token: &str) -> String {
    format!("t={}; SameSite=Lax; Path=/", token)
}

// ──────────────────────────────────────────────
// Router
// ──────────────────────────────────────────────

pub fn tachyon_router(state: TachyonState) -> Router<TachyonState> {
    Router::new()
        .route("/test", get(serve_index))
        .route("/verify", get(get_verify))
        .route("/verify/recovery_key", post(verify_recovery_key))
        .route("/webauth/nfy", get(get_webauth_nfy))
        .layer(middleware::from_fn_with_state(state.clone(), is_authenticated))
        .route("/", get(serve_index))
        .route("/webauth", get(get_webauth))
        .route("/webauth", post(post_webauth))
        .route("/auth", get(get_auth))
        .route("/auth", post(post_auth))
        .route("/{file}", get(serve_static))
        .layer(middleware::from_fn(extract_token))
        .with_state(state)
}

// ──────────────────────────────────────────────
// Middleware
// ──────────────────────────────────────────────

async fn is_authenticated(
    State(state): State<TachyonState>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    if let Some(token) = req.extensions().get::<String>() {
        if state.tachyon_clients().get(token).is_some() {
            return next.run(req).await;
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(SET_COOKIE, clear_token_cookie())
        .body(index_page(login_form()).into_string().into())
        .unwrap()
}

async fn extract_token(req: Request<Body>, next: Next) -> impl IntoResponse {
    let mut req = req;

    let mut token = None;
    let mut should_set_cookie = false;

    if let Some(t) = extract_token_from_query(&req) {
        token = Some(t);
        should_set_cookie = true;
    } else if let Some(t) = extract_token_from_cookie(&req) {
        token = Some(t);
    }

    if let Some(token_value) = token {
        req.extensions_mut().insert(token_value.clone());
        let mut response = next.run(req).await;
        if should_set_cookie {
            response.headers_mut().insert(
                SET_COOKIE,
                set_token_cookie(&token_value).parse().unwrap(),
            );
        }
        response
    } else {
        next.run(req).await
    }
}

fn extract_token_from_cookie(req: &Request<Body>) -> Option<String> {
    let cookie_header = req.headers().get(COOKIE)?;
    let cookie_str = cookie_header.to_str().ok()?;
    let cookie = cookie_str
        .split(';')
        .find(|s| s.trim().starts_with("t="))?;
    let value = cookie.trim().strip_prefix("t=")?.to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn extract_token_from_query(req: &Request<Body>) -> Option<String> {
    let _query = req.uri().query()?;
    let params: Params = axum::extract::Query::try_from_uri(req.uri())
        .map(|axum::extract::Query(params)| params)
        .ok()?;
    let t = params.get("t")?;
    // Validate it's a valid header value
    axum::http::HeaderValue::from_str(t).ok()?;
    Some(t.clone())
}

// ──────────────────────────────────────────────
// Handlers
// ──────────────────────────────────────────────

async fn serve_index() -> Html<String> {
    Html(
        index_page(html! {
            h2 { "Tachyon is running..." }
        })
            .into_string(),
    )
}

async fn get_auth(axum::extract::Query(params): axum::extract::Query<Params>) -> Html<String> {
    let username = get_param(&params, "username").unwrap_or("");

    let content = html! {
        form method="POST" ic-post-to="/tachyon/auth" ic-target=".content" {
            input type="text" name="username" placeholder="Username" value=(username);
            input type="password" name="password" placeholder="Password";
            button type="submit" { "Log In" }
        }
    };

    Html(index_page(content).into_string())
}

async fn post_auth(
    State(state): State<TachyonState>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {
    let username = get_param(&form_data, "username").unwrap_or("Unknown");
    let password = get_param(&form_data, "password").unwrap_or("");

    let email = EmailAddress::from_str(username).unwrap();
    let matrix_id = email.to_owned_user_id();

    let login_successful =
        if let Ok((matrix_token, _)) = login_with_password(matrix_id, password, true).await {
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

async fn post_webauth(
    State(state): State<TachyonState>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {
    let username = get_param(&form_data, "email").unwrap_or("Unknown");
    let email = EmailAddress::from_str(username).unwrap();

    let Some(client) = state.tachyon_clients().find_by_email(&email) else {
        let page = html! {
            div class="container" {
                h2 { "Tachyon Web Login" }
                p { "Could not find a logged in client for " (username) "." }
            }
        };
        return Html(page.into_string());
    };

    let user = client.own_user();
    let token = client.ticket_token().0;
    let notification_id: i32 = rand::random();

    let (alert, recv) = Alert::new(AlertType::WebLoginRequest);
    client.alerts().insert(notification_id, alert);
    state.store_pending_alert(notification_id, recv);

    let nfy_url = format!(
        "http://127.0.0.1:8080/tachyon/webauth/nfy?t={}&notification_id={}&email={}",
        &token, notification_id, username
    );

    let secret_not = NotificationServerCommand::NOT(NotServer {
        payload: NotificationPayloadType::Normal(NotificationFactory::alert(
            &user.uuid,
            user.get_email_address(),
            "Login request to Tachyon Web",
            "http://127.0.0.1:8080/tachyon",
            &nfy_url,
            &nfy_url,
            None,
            notification_id,
        )),
    });
    let _ = client.notification_handle().send(secret_not).await;

    Html(webauth_poll_snippet(notification_id, username).into_string())
}

async fn get_webauth_nfy(
    State(state): State<TachyonState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Html<String> {
    let notification_id = parse_notification_id(&params).unwrap();

    let client = state.tachyon_clients().get(&token).unwrap();
    let (_id, mut alert) = client.alerts().remove(&notification_id).unwrap();
    alert.notify_success();

    Html(
        index_page(html! {
            h2 { "Tachyon Web Login" }
            p { "You have been logged in." }
        })
            .into_string(),
    )
}

async fn get_webauth(
    State(state): State<TachyonState>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> impl IntoResponse {
    let notification_id = parse_notification_id(&params).unwrap();
    let username = require_param(&params, "email").unwrap();

    if let Some(mut recv) = state.take_pending_alert(&notification_id) {
        match recv.try_recv() {
            Ok(crate::tachyon::tachyon_client::AlertResult::AlertSuccess) => {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("X-IC-Redirect", "/tachyon")
                    .body(Body::from("Logged in! Redirecting..."))
                    .unwrap();
            }
            Ok(crate::tachyon::tachyon_client::AlertResult::AlertFailure) => {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("X-IC-Redirect", "/tachyon")
                    .body(Body::from("Not Logged in! Redirecting..."))
                    .unwrap();
            }
            Err(_) => {}
        }

        state.store_pending_alert(notification_id, recv);
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            webauth_poll_snippet(notification_id, username).into_string(),
        ))
        .unwrap()
}

async fn get_verify(
    State(state): State<TachyonState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Html<String> {
    let notification_id = parse_notification_id(&params).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let _notification = tachyon_client.alerts().get(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    let recover_enabled = matrix_client
        .encryption()
        .secret_storage()
        .is_enabled()
        .await
        .unwrap();

    let content = html! {
        h2 { "Verify your device" }
        @if recover_enabled {
            p { "You can use your recovery key to verify this device." }
            form method="POST" ic-post-to="/tachyon/verify/recovery_key" ic-target=".content" enctype="multipart/form-data" {
                input type="file" name="recovery_key" accept=".txt" required;
                input type="hidden" name="notification_id" value=(notification_id) required;
                button type="submit" { "Verify Device" }
            }
        } @else {
            p { "It looks like you don't have a recovery key set up." }
            button type="button" ic-post-to="/tachyon/verify" ic-target=".content" { "Reset Identity" }
        }
    };

    Html(index_page(content).into_string())
}

async fn verify_recovery_key(
    State(state): State<TachyonState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    mut multipart: Multipart,
) -> Html<String> {
    let mut notification_id = None;
    let mut recovery_key = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        match field.name() {
            Some("notification_id") => {
                notification_id = Some(i32::from_str(&field.text().await.unwrap()).unwrap());
            }
            Some("recovery_key") => {
                recovery_key = Some(field.text().await.unwrap());
            }
            _ => {}
        }
    }

    let notification_id = notification_id.unwrap();
    let recovery_key = recovery_key.unwrap().replace(" ", "");

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let (_id, mut alert) = tachyon_client.alerts().remove(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    println!("Recovering device with recovery key: {}", recovery_key);

    let store = matrix_client
        .encryption()
        .secret_storage()
        .open_secret_store(recovery_key.trim())
        .await
        .unwrap();
    store.import_secrets().await.unwrap();

    let status = matrix_client
        .encryption()
        .cross_signing_status()
        .await
        .expect("We should be able to check our cross-signing status");

    matrix_client
        .encryption()
        .get_own_device()
        .await
        .unwrap()
        .unwrap()
        .verify()
        .await
        .unwrap();

    let successful = status.is_complete();

    if successful {
        alert.notify_success();
    } else {
        alert.notify_failure();
    }

    let page = html! {
        h2 { "Verification Result" }
        @if successful {
            p { "Your device is now verified!" }
        } @else {
            p { "Your device could not be verified :(" }
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
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap()
        }
    };

    Response::builder()
        .header(CONTENT_TYPE, content_type)
        .header(CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(data))
        .expect("response to be valid")
}