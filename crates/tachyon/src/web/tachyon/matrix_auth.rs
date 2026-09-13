use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::{layout, Params};
use axum::body::Body;
use axum::extract::State;
use axum::http::header::LOCATION;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use log::{debug, error, warn};
use maud::{html, Markup};
use tachyon_core::application::auth_use_case::FinishedLogin;
use tachyon_core::application::error::AuthError;
use tachyon_core::domain::auth::{Credential, InteractiveAuthStarted};
use tachyon_core::domain::verification::Password;

/// Where the `NOT` alert sent during sign-in lands.
///
/// The client only carries a flow id, so the URL the user actually needs, which for OAuth
/// is long, query-heavy and generated per attempt, is looked up here rather than shipped
/// through MSNP.
pub async fn get_login_start(
    State(state): State<GlobalState>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Response {
    let Some(flow_id) = params.get("flow") else {
        return error_page("This login link is missing its flow id.");
    };

    match state.app_state().auth_use_case().prompt(flow_id) {
        Some(InteractiveAuthStarted::OAuth { auth_url, .. }) => redirect(&auth_url),
        Some(InteractiveAuthStarted::PasswordRequired) => password_page(flow_id, None),
        None => error_page("This login has expired or was already completed."),
    }
}

/// The password form posts here. A rejected password shows the form again; anything else
/// that goes wrong ends the sign-in.
pub async fn post_login_password(
    State(state): State<GlobalState>,
    axum::extract::Form(form): axum::extract::Form<Params>,
) -> Response {
    let Some(flow_id) = form.get("flow") else {
        return error_page("This login form is missing its flow id.");
    };
    let Some(password) = form.get("password").filter(|password| !password.is_empty()) else {
        return password_page(flow_id, Some("Please fill in your password."));
    };

    let credential = Credential::Password(Password::new(password.as_str()));
    match state
        .app_state()
        .auth_use_case()
        .finish_login(flow_id, credential)
        .await
    {
        Ok(finished) => login_finished(finished),
        Err(AuthError::BackendError(e)) => {
            warn!("The homeserver rejected the password login: {:?}", e);
            password_page(flow_id, Some("Your homeserver did not accept that password."))
        }
        Err(AuthError::LoginNotFound) => {
            error_page("This login has expired or was already completed.")
        }
        Err(e) => {
            error!("Could not finish the password login: {:?}", e);
            error_page("The login succeeded but could not be stored.")
        }
    }
}

/// Where the backend sends the browser back once the user has authorized.
pub async fn get_login_callback(
    State(state): State<GlobalState>,
    request: axum::extract::Request,
) -> Response {
    // The SDK wants the whole query string, not just the code, and it needs the `state`
    // parameter out of it to find its way back to this login.
    let Some(query) = request.uri().query().map(str::to_owned) else {
        return error_page("The login callback carried no parameters.");
    };

    let params: Params = match axum::extract::Query::try_from_uri(request.uri()) {
        Ok(axum::extract::Query(params)) => params,
        Err(_) => return error_page("The login callback parameters could not be read."),
    };

    let Some(flow_id) = params.get("state") else {
        return error_page("The login callback carried no state parameter.");
    };

    let auth_use_case = state.app_state().auth_use_case();

    if let Some(error) = params.get("error") {
        warn!("Authorization was refused by the homeserver: {}", error);
        if let Err(e) = auth_use_case.abandon_flow(flow_id).await {
            error!("Could not abandon the refused login: {:?}", e);
        }
        return error_page("Your homeserver refused the authorization.");
    }

    match auth_use_case
        .finish_login(flow_id, Credential::OAuthCallback(query))
        .await
    {
        Ok(finished) => login_finished(finished),
        Err(e) => {
            error!("Could not finish the interactive login: {:?}", e);
            error_page("Your homeserver rejected the login.")
        }
    }
}

fn login_finished(finished: FinishedLogin) -> Response {
    match finished.next_url {
        None => {
            debug!("Interactive login finished");
            success_page()
        }
        // The user is already here, so send them straight on rather than making them come
        // back through a second alert.
        Some(next_url) => {
            warn!("Interactive login finished but its device is unverified");
            redirect(&next_url)
        }
    }
}

/// 303 rather than 307: the password form arrives as a POST, and the browser must follow
/// with a GET or the confirmation page answers 405.
fn redirect(location: &str) -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(LOCATION, location)
        .body(Body::empty())
        .unwrap()
}

fn success_page() -> Response {
    let content = html! {
        div class="container" {
            h2 { "Signed in" }
            p { "You can go back to Messenger now." }
        }
    };

    Html(layout::tachyon_page_no_nav(content).into_string()).into_response()
}

fn password_page(flow_id: &str, error: Option<&str>) -> Response {
    let content = html! {
        div class="container" {
            h2 { "Sign in to your Matrix account" }
            p { "Your homeserver does not offer single sign-on, so it needs your password." }
            @if let Some(error) = error {
                p class="error" { (error) }
            }
            form action="/tachyon/login/password" method="POST" {
                input type="hidden" name="flow" value=(flow_id) {}
                label for="password" { "Password" }
                br;
                input type="password" name="password" id="password" autofocus {}
                br;
                input type="submit" value="Sign in" {}
            }
        }
    };

    Html(layout::tachyon_page_no_nav(content).into_string()).into_response()
}

fn error_page(message: &str) -> Response {
    let content = error_markup(message);
    Html(layout::tachyon_page_no_nav(content).into_string()).into_response()
}

fn error_markup(message: &str) -> Markup {
    html! {
        div class="container" {
            h2 { "Sign-in problem" }
            p { (message) }
            p { "Sign out of Messenger and sign in again to start over." }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tachyon_core::domain::auth::TachyonToken;
    use tachyon_core::domain::verification::DeviceStatus;

    #[test]
    fn an_untrusted_device_is_sent_to_confirmation_with_a_get() {
        let response = login_finished(FinishedLogin {
            token: TachyonToken::new("ticket"),
            device_status: DeviceStatus::Unverified,
            next_url: Some("http://127.0.0.1:11866/tachyon/confirm_device?t=ticket".to_string()),
        });

        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(
            response.headers().get(LOCATION).unwrap(),
            "http://127.0.0.1:11866/tachyon/confirm_device?t=ticket"
        );
    }
}
