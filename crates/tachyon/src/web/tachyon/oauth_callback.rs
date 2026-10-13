use axum::extract::{Request, State};
use axum::http::header::HOST;
use axum::response::Response;
use matrix_sdk::utils::UrlOrQuery;

use crate::matrix::services::login::persist_session;
use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::login::{error_page, redirect_to_next_step};
use reqwest::Url;

// mounted at the web root, not under /tachyon: the homeserver's OAuth redirect URI points here
pub(crate) async fn oauth_callback(State(state): State<GlobalState>, req: Request) -> Response {
    let callback_url = rebuild_callback_url(&state, &req);

    let params: crate::web::tachyon::Params = axum::extract::Query::try_from_uri(req.uri())
        .map(|axum::extract::Query(params)| params)
        .unwrap_or_default();

    let Some(state_param) = params.get("state") else {
        return oauth_error("The OAuth callback is missing its state parameter.");
    };

    let Some((token, _pending)) = state.find_pending_login_by_oauth_state(state_param) else {
        return oauth_error("This login link has expired or was already used. Reopen Tachyon from the Messenger alert and try again.");
    };
    let Some(oauth) = state.take_pending_oauth(&token) else {
        return oauth_error("This login was already completed. Reopen Tachyon from the Messenger alert.");
    };

    let callback = match Url::parse(&callback_url) {
        Ok(url) => UrlOrQuery::Url(url),
        Err(error) => return oauth_error(&format!("Invalid OAuth callback URL `{}`: {}", callback_url, error)),
    };

    if let Err(error) = oauth.client.oauth().finish_login(callback).await {
        log::warn!("OAuth finish_login failed: {}", error);
        return oauth_error(&format!("Login could not be completed: {}", error));
    }

    persist_session(&oauth.client, &token, state.credentials());
    state.finish_login(&token, oauth.client).await;
    redirect_to_next_step(&state, &token, req.headers()).await
}

fn rebuild_callback_url(state: &GlobalState, req: &Request) -> String {
    let host = req
        .headers()
        .get(HOST)
        .and_then(|host| host.to_str().ok())
        .map(str::to_string)
        .unwrap_or_else(|| format!("127.0.0.1:{}", state.get_config().http_port));
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|path| path.as_str())
        .unwrap_or("/oauth/callback");
    format!("http://{}{}", host, path_and_query)
}

// outside the intercooler wrapper: render the full page here
fn oauth_error(message: &str) -> Response {
    error_page(message)
}
