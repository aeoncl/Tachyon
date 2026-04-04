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
use maud::{html, Markup, DOCTYPE};
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

async fn is_authenticated(
    State(state): State<TachyonState>,
    mut req: Request<Body>,
    next: Next
) -> impl IntoResponse {

    if let Some(token) = req.extensions().get::<String>().cloned() {
        if let Some(_) = state.tachyon_clients().get(&token) {
            return next.run(req).await;
        }
    }

    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(SET_COOKIE, "t=; Max-Age=-1; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax;")
        .body(index_page(
            html! {
                h2 { "Tachyon Web Login" }
                p { "If your client is logged in, fill in your email address and you will receive a notification." }
                form method="POST" ic-post-to="/tachyon/webauth" ic-target=".content" {
                    input type="text" name="email" placeholder="Email";
                    button type="submit" { "Request Login" }
                }
            }
        ).into_string().into())
        .unwrap()

}


async fn extract_token(
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    let mut req = req;

    let mut token = None;
    let mut insert = false;

      if let Some(_query) = req.uri().query() {
        let params: std::collections::HashMap<String, String> = axum::extract::Query::try_from_uri(&req.uri())
            .map(|axum::extract::Query(params)| params)
            .unwrap_or_default();

        if let Some(t) = params.get("t") {
            if let Ok(header_value) = axum::http::HeaderValue::from_str(t) {
                token = Some(header_value.to_str().unwrap().to_string());
                insert = true;
            }
        }
      }

    if let Some(cookie_header) = req.headers().get(COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            if let Some(cookie) = cookie_str.split(';').find(|s| s.trim().starts_with("t=")) {
                let value =  cookie.trim()["t=".len()..].to_string();
                if !value.is_empty() {
                    token = Some(value);
                    insert = false;
                }
            }
        }
    }


    if let Some(token_value) = token {
        req.extensions_mut().insert(token_value.clone());
        let mut response = next.run(req).await;
        if insert {
            response.headers_mut().insert(SET_COOKIE, format!("t={}; SameSite=Lax; Path=/", token_value).parse().unwrap());
        }
        response
    } else {
        next.run(req).await
    }

}

fn error_page(message: String) -> Markup {
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

                        }
                    }
                }

                div class="content" {
                    h2 { "Something went wrong" }
                    p { (message) }
                }
            }
        }
    }
}

pub fn index_page(content: Markup) -> Markup {
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

                div class="content" {
                    (content)
                }
            }
        }
    }
}

async fn serve_index() -> Html<String> {
    Html(index_page(html! {
        h2 { "Tachyon is running..." }
    }).into_string())
}

async fn get_auth(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> Html<String> {

    let username = params.get("username").map(|s| s.as_str()).unwrap_or("");

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

async fn post_webauth(
    State(state): State<TachyonState>,
    axum::extract::Form(form_data): axum::extract::Form<std::collections::HashMap<String, String>>
) -> Html<String> {

    let username = form_data.get("email").map(|s| s.as_str()).unwrap_or("Unknown");

    let email = EmailAddress::from_str(&username).unwrap();

    let matrix_id = email.to_owned_user_id();

    let found = state.tachyon_clients().find_by_email(&email);

    match found {
        None => {
            let page = html! {
                div class="container"  {
                    h2 { "Tachyon Web Login" }
                    p { "Could not find a logged in client for " (username) "." }
                }
            };

            return Html(page.into_string());        }
        Some(client) => {

            let user = client.own_user();
            let token = client.ticket_token().0;

            let notification_id: i32 = rand::random();

            let (alert, recv) = Alert::new(AlertType::WebLoginRequest);
            client.alerts().insert(notification_id, alert);

            state.store_pending_alert(notification_id, recv);


            let secret_not = NotificationServerCommand::NOT(NotServer {
                payload: NotificationPayloadType::Normal(NotificationFactory::alert(&user.uuid, user.get_email_address(), "Login request to Tachyon Web", "http://127.0.0.1:8080/tachyon", format!("http://127.0.0.1:8080/tachyon/webauth/nfy?t={}&notification_id={}&email={}", &token, notification_id, username).as_str(), format!("http://127.0.0.1:8080/tachyon/webauth/nfy?t={}&notification_id={}&email={}", &token, notification_id, username).as_str(), None, notification_id)),
            });
            let _ = client.notification_handle().send(secret_not).await;

            let page = html! {
                div class="container" ic-poll="2s" ic-src={"/tachyon/webauth?notification_id=" (notification_id) "&email=" (username)} {
                    h2 { "Tachyon Web Login" }
                    p { "A notification has been sent to your client. Please approve the login." }
                }
            };

            return Html(page.into_string());
        }
    }
}

async fn get_webauth_nfy(
    State(state): State<TachyonState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> Html<String> {

    let notification_id = i32::from_str(params.get("notification_id").map(|s| s.as_str()).unwrap()).unwrap();
    let username = params.get("email").map(|s| s.as_str()).unwrap();

    let email = EmailAddress::from_str(&username).unwrap();
    let matrix_id = email.to_owned_user_id();

    let client = state.tachyon_clients().get(&token).unwrap();
    let (id, mut alert) = client.alerts().remove(&notification_id).unwrap();
    alert.notify_success();

    Html(index_page(html! {
        h2 { "Tachyon Web Login" }
        p { "You have been logged in." }
    }).into_string())

}

async fn get_webauth(
    State(state): State<TachyonState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> impl IntoResponse {

    let notification_id = i32::from_str(params.get("notification_id").map(|s| s.as_str()).unwrap()).unwrap();
    let username = params.get("email").map(|s| s.as_str()).unwrap();

    let email = EmailAddress::from_str(&username).unwrap();

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
            Err(err) => {}
        }

    }

    let page = html! {
                div class="container" ic-poll="2s" ic-src={"/tachyon/webauth?notification_id=" (notification_id) "&email=" (username)} {
                    h2 { "Tachyon Web Login" }
                    p { "A notification has been sent to your client. Please approve the login." }
                }
            };

    return Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(page.into_string()))
        .unwrap();
}

async fn get_verify(
    State(state): State<TachyonState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>
) -> Html<String> {

    //TODO error handling
    let notification_id = i32::from_str(params.get("notification_id").map(|s| s.as_str()).unwrap()).unwrap();

    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let notification = tachyon_client.alerts().get(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    let recover_enabled =  matrix_client.encryption().secret_storage().is_enabled().await.unwrap();

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
    mut multipart: Multipart
) -> Html<String> {

    let mut notification_id = None;
    let mut recovery_key = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(name) = field.name() {
            if name == "notification_id" {
                notification_id = Some(i32::from_str(&*field.text().await.unwrap()).unwrap());
            } else if name == "recovery_key" {
                recovery_key = Some(field.text().await.unwrap());
            }
        }
    }

    let notification_id = notification_id.unwrap();
    let recovery_key = recovery_key.unwrap().replace(" ", "");


    let tachyon_client = state.tachyon_clients().get(&token).unwrap();
    let (id, mut alert) = tachyon_client.alerts().remove(&notification_id).unwrap();
    let matrix_client = state.matrix_clients().get(&token).unwrap();

    println!("Recovering device with recovery key: {}", recovery_key);

    let store = matrix_client.encryption().secret_storage().open_secret_store(recovery_key.trim()).await.unwrap();
    store.import_secrets().await.unwrap();

    let status = matrix_client
        .encryption()
        .cross_signing_status()
        .await
        .expect("We should be able to check out cross-signing status");


    matrix_client.encryption().get_own_device().await.unwrap().unwrap().verify().await.unwrap();

    let status_succesful = status.is_complete();

    if status_succesful {
        alert.notify_success();
    } else {
        alert.notify_failure();
    }

    let page = html! {
        h2 { "Verification Result" }
        @if status_succesful {
            p { "Your device is now verified !" }
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
        _ => return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    };

    Response::builder()
        .header(CONTENT_TYPE, content_type)
        .header(CACHE_CONTROL, "public, max-age=31536000")
        .body(Body::from(data))
        .expect("response to be valid")
}