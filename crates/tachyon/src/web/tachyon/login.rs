use std::str::FromStr;
use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::http::{Request, Response, StatusCode};
use axum::body::Body;
use axum::http::header::{LOCATION, SET_COOKIE};
use maud::{html, Markup};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::not::{NotServer, NotificationPayloadType};
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::ticket_token::TicketToken;
use crate::tachyon::alert::{Alert, AlertError, AlertNotify, AlertSuccess};
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::{layout, login, Params};
use crate::web::tachyon::middleware::set_token_cookie;

const TITLE: &str = "Tachyon Web Login";

pub async fn post_login_request(
    State(state): State<GlobalState>,
    axum::extract::Form(form_data): axum::extract::Form<Params>,
) -> Html<String> {
    let username = form_data.get("email").map(|s| s.as_str()).unwrap_or("Unknown");
    let email = EmailAddress::from_str(username).unwrap();

    let Some(client) = state.tachyon_clients().find_by_email(&email) else {
        let page = html! {
            div class="container" {
                h2 { (TITLE) }
                p { "Could not find a logged in client for " (username) "." }
            }
        };
        return Html(page.into_string());
    };

    let user = client.own_user();
    let token = client.ticket_token().0;
    let notification_id: i32 = rand::random();

    let (alert, recv) = Alert::new_weblogin();
    client.alerts().insert(notification_id, alert);
    state.store_pending_alert(notification_id, recv);

    let nfy_url = format!(
        "http://127.0.0.1:8080/tachyon/login/nfy?t={}&notification_id={}&email={}",
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

    Html(login_poll_snippet(notification_id, username).into_string())
}

pub async fn get_login_nfy(
    State(state): State<GlobalState>,
    axum::extract::Extension(token): axum::extract::Extension<String>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> impl IntoResponse  {

    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let client = state.tachyon_clients().get(&token).unwrap();
    let (_id, mut alert) = client.alerts().remove(&notification_id).unwrap();
    alert.notify_success(AlertSuccess::TicketToken(TicketToken(token)));

    Response::builder()
        .status(StatusCode::PERMANENT_REDIRECT)
        .header(LOCATION, "/tachyon")
        .body(Body::empty())
        .unwrap()
}

pub async fn get_login_request(
    State(state): State<GlobalState>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> impl IntoResponse {

    let notification_id_str = params.get("notification_id").map(|s| s.as_str()).unwrap_or_default();
    let notification_id = i32::from_str(notification_id_str).map_err(|e| format!("Invalid notification_id: {}", e)).unwrap();

    let email_str = params.get("email").map(|s| s.as_str()).unwrap_or_default();
    let email = EmailAddress::from_str(email_str).map_err(|e| format!("Invalid email address: {}", e)).unwrap();

    if let Some(mut recv) = state.take_pending_alert(&notification_id) {

        match recv.try_recv() {
            Ok(Some(AlertSuccess::TicketToken(token))) => {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("X-IC-Redirect", "/tachyon")
                    .header(SET_COOKIE, set_token_cookie(token.as_str()))
                    .body(Body::from("Logged in! Redirecting..."))
                    .unwrap();
            }
            Err(err) => {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("X-IC-Redirect", "/tachyon/login")
                    .body(Body::from("Not Logged in! Redirecting..."))
                    .unwrap();
            }
            _ => {}
        }

        state.store_pending_alert(notification_id, recv);
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            login_poll_snippet(notification_id, email.as_str()).into_string(),
        ))
        .unwrap()
}

fn login_poll_snippet(notification_id: i32, email: &str) -> Markup {
    html! {
        div class="container" ic-poll="2s" ic-src={"/tachyon/login/request?notification_id=" (notification_id) "&email=" (email)} {
            h2 { "Tachyon Web Login" }
            p { "A notification has been sent to your client. Please approve the login." }
        }
    }
}

fn login_form() -> Markup {
    html! {
        h2 { (TITLE) }
        p { "If your client is logged in, fill in your email address and you will receive a notification." }
        form method="POST" ic-post-to="/tachyon/login/request" ic-target=".content" {
            input type="text" name="email" placeholder="Email";
            button type="submit" { "Request Login" }
        }
    }
}

pub async fn get_login_page(
    req: Request<Body>,
) -> impl IntoResponse {

    if let Some(_) = req.extensions().get::<String>() {
        return Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/tachyon")
            .body(Body::empty())
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            layout::tachyon_page_no_nav(login::login_form()).into_string(),
        ))
        .unwrap()


}