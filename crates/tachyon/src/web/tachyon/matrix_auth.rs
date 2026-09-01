use crate::matrix::cross_signing::check_device_is_crossed_signed;
use log::{debug, error, warn};
use crate::tachyon::alert::{AlertNotify, AlertSuccess};
use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::{layout, Params};
use axum::body::Body;
use axum::extract::State;
use axum::http::header::LOCATION;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use maud::{html, Markup};
use tachyon_backend_matrix::infrastructure::backend::BackendSessionMatrix;
use tachyon_core::domain::auth::InteractiveAuthStarted;

/// Where the `NOT` alert sent during sign-in lands.
///
/// The client only carries a flow id, so the URL the user actually needs — which for OAuth
/// is long, query-heavy and generated per attempt — is looked up here rather than shipped
/// through MSNP.
pub async fn get_login_start(
    State(state): State<GlobalState>,
    axum::extract::Query(params): axum::extract::Query<Params>,
) -> Response {
    let Some(flow_id) = params.get("flow") else {
        return error_page("This login link is missing its flow id.");
    };

    let auth_url = state.peek_pending_login(flow_id, |pending| match &pending.prompt {
        InteractiveAuthStarted::OAuth { auth_url, .. } => Some(auth_url.clone()),
        InteractiveAuthStarted::PasswordRequired => None,
    });

    match auth_url {
        Some(Some(auth_url)) => Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, auth_url)
            .body(Body::empty())
            .unwrap(),
        // The login exists but the homeserver has no OAuth.
        Some(None) => password_not_supported_page(),
        None => error_page("This login has expired or was already completed."),
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

    if let Some(error) = params.get("error") {
        warn!("Authorization was refused by the homeserver: {}", error);
        if let Some(pending) = state.take_pending_login(flow_id) {
            let _ = pending
                .alert
                .notify_failure(anyhow::anyhow!("Authorization was refused: {}", error));
        }
        return error_page("Your homeserver refused the authorization.");
    }

    let Some(pending) = state.take_pending_login(flow_id) else {
        return error_page("This login has expired or was already completed.");
    };

    let auth_use_case = state.app_state().auth_use_case();

    let session = match auth_use_case
        .finish_interactive_login(&pending.login_id, &query)
        .await
    {
        Ok(session) => session,
        Err(e) => {
            error!("Could not finish the interactive login: {:?}", e);
            let _ = pending
                .alert
                .notify_failure(anyhow::anyhow!("Could not finish login: {:?}", e));
            return error_page("Your homeserver rejected the login.");
        }
    };
    
    let ticket = state.ticket_for(&pending.email);
    if let Err(e) = auth_use_case
        .bind_token(state.token_for(&pending.email), pending.login_id.clone())
        .await
    {
        error!("Could not link the ticket to the login: {:?}", e);
        let _ = pending
            .alert
            .notify_failure(anyhow::anyhow!("Could not store the login: {:?}", e));
        return error_page("The login succeeded but could not be stored.");
    }

    // TEMPORARY (refactor scaffold): device confirmation still drives matrix-sdk directly.
    let Some(matrix_client) = session
        .as_any()
        .downcast_ref::<BackendSessionMatrix>()
        .map(|session| session.matrix_client().clone())
    else {
        let _ = pending
            .alert
            .notify_failure(anyhow::anyhow!("Backend session is not a matrix session"));
        return error_page("The login succeeded but the session was of an unexpected kind.");
    };

    let cross_signed = match check_device_is_crossed_signed(&matrix_client).await {
        Ok(cross_signed) => cross_signed,
        Err(e) => {
            // Treated as unconfirmed: sending the user through confirmation is the safe
            // reading, and it surfaces the problem rather than signing in regardless.
            warn!("Could not check whether the device is cross signed: {}", e);
            false
        }
    };

    debug!(
        "Interactive login finished for {}, device cross signed: {}",
        pending.email.as_str(),
        cross_signed
    );

    if !cross_signed {
        // The user is already here, so send them straight on to confirm the device rather
        // than making them come back through a second alert. The login alert is handed to
        // the confirmation pages and fires once they are done, which is also what keeps the
        // MSNP client from connecting with an unverified device.
        let notification_id = rand::random::<i32>();
        state.insert_pre_session(
            ticket.as_str().to_owned(),
            matrix_client,
            notification_id,
            pending.alert,
        );

        return Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(
                LOCATION,
                format!(
                    "/tachyon/confirm_device?t={}&notification_id={}",
                    ticket.as_str(),
                    notification_id
                ),
            )
            .body(Body::empty())
            .unwrap();
    }

    // Releases the USR handler that is holding the client's sign-in open.
    let _ = pending.alert.notify_success(AlertSuccess::Unit);

    success_page(pending.email.as_str())
}

fn success_page(email: &str) -> Response {
    let content = html! {
        div class="container" {
            h2 { "Signed in" }
            p { "You are signed in as " (email) "." }
            p { "You can go back to Messenger now." }
        }
    };

    Html(layout::tachyon_page_no_nav(content).into_string()).into_response()
}

fn password_not_supported_page() -> Response {
    let content = html! {
        div class="container" {
            h2 { "Password sign-in is not supported yet" }
            p {
                "Your homeserver does not offer OAuth, so Tachyon would have to ask you for
                 your password directly. That path has not been wired up yet."
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
