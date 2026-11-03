use axum::body::Body;
use axum::extract::{Extension, Form, State};
use axum::http::header::LOCATION;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use maud::{html, Markup};
use tachyon_sqlite_store::HomeserverUrl;

use crate::matrix::cross_signing::check_device_is_crossed_signed;
use crate::matrix::services::homeserver::{ResolutionError, ResolvedHomeserver};
use crate::matrix::services::login::{get_matrix_client_builder, persist_session};
use crate::tachyon::client::tachyon_client::PendingOAuth;
use crate::tachyon::config::paths::get_store_path;
use crate::tachyon::global_state::{GlobalState};
use crate::tachyon::mappers::user_id::MatrixIdCompatible;
use crate::web::tachyon::layout;
use crate::web::tachyon::Params;

const TITLE: &str = "Log in";

pub async fn get_login_page(
    State(state): State<GlobalState>,
    Extension(token): Extension<String>,
    axum::extract::Query(query): axum::extract::Query<Params>,
) -> Response {
    if state.get_clients(&token).map_or(false, |client| client.is_backend_ready()) {
        return redirect(&format!("/tachyon?t={}", token), &HeaderMap::new());
    }

    let (username, homeserver) = prefill(&state, &token, &query);

    let checked = state.homeserver_resolver().resolve(&homeserver, state.get_config().allow_plain_http).await.ok();
    let prefilled = checked
        .as_ref()
        .map(|resolved| resolved.url.to_string())
        .unwrap_or_else(|| format!("https://{homeserver}"));
    let auth = auth_area(&token, checked.as_ref());

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(
            layout::tachyon_page_no_nav(login_form(&token, &username, &prefilled, None, auth)).into_string(),
        ))
        .unwrap()
}

pub async fn get_login_check(
    State(state): State<GlobalState>,
    Extension(token): Extension<String>,
    axum::extract::Query(query): axum::extract::Query<Params>,
) -> Response {
    let url = query
        .get("homeserver_url")
        .or_else(|| query.get("url"))
        .cloned()
        .unwrap_or_default();
    let auth = match state.homeserver_resolver().resolve(&url, state.get_config().allow_plain_http).await {
        Ok(resolved) => auth_area(&token, Some(&resolved)),
        Err(error) => auth_area_error(&token, Some(&error)),
    };
    Html(auth.into_string()).into_response()
}

pub async fn post_login(
    State(state): State<GlobalState>,
    Extension(token): Extension<String>,
    headers: HeaderMap,
    Form(form): Form<Params>,
) -> Response {
    let Some(client) = state.get_clients(&token) else {
        return error_page("Unknown or expired session — reopen Tachyon from the Messenger alert.");
    };
    if client.is_backend_ready() {
        return redirect_to_next_step(&state, &token, &headers).await;
    }

    let username = form.get("username").cloned().unwrap_or_default();
    let homeserver = form.get("homeserver_url").cloned().unwrap_or_default();
    let sso = form.get("action").map(String::as_str) == Some("sso");

    let resolved = match state.homeserver_resolver().resolve(&homeserver, state.get_config().allow_plain_http).await {
        Ok(resolved) => resolved,
        Err(error) => return login_error_page(&token, &username, &homeserver, &error),
    };

    let matrix_id = client.own_user().get_email_address().to_owned_user_id();

    if sso {
        if !resolved.supports_oauth {
            return login_error_message(&token, &username, &homeserver, "This homeserver does not support SSO login. Use the password form.");
        }
        return match begin_sso_login(&state, &token, &matrix_id, &resolved.url).await {
            Ok(url) => redirect(&url, &headers),
            Err(error) => login_error_message(&token, &username, &homeserver, &format!("Login failed. Please try again. ({error})")),
        };
    }

    let password = form.get("password").cloned().unwrap_or_default();
    // the credentials row is kept on a dead token: the re-login reuses its device id
    let device_id = state.credentials().load(&token).ok().flatten().map(|credentials| credentials.device_id().clone());

    let login = state
        .matrix_login_service()
        .login_with_password(&matrix_id, &password, device_id.as_ref(), Some(&resolved.url), !state.get_config().strict_ssl)
        .await;

    match login {
        Ok(matrix_client) => {
            persist_session(&matrix_client, &token, state.credentials());
            client.attach_backend(matrix_client, state.credentials_handle());
            redirect_to_next_step(&state, &token, &headers).await
        }
        Err(_) => login_error_message(&token, &username, &homeserver, "Login failed. Please try again."),
    }
}

async fn begin_sso_login(state: &GlobalState, token: &str, matrix_id: &matrix_sdk::ruma::UserId, homeserver_url: &HomeserverUrl) -> Result<String, anyhow::Error> {
    // A prior login already set the SDK's auth data; registering a client on top panics inside the SDK.
    let client = get_matrix_client_builder(matrix_id.server_name(), Some(homeserver_url), !state.get_config().strict_ssl)
        .sqlite_store(get_store_path(matrix_id), None)
        .build()
        .await?;

    let oauth = client.oauth();
    let (raw_metadata, redirect_uri) = oauth_registration_metadata(state.get_config().http_port)?;

    if oauth.client_id().is_none() {
        log::info!("registering OAuth client with {}", homeserver_url);
        oauth.register_client(&raw_metadata).await?;
    }

    let authorization = oauth
        .login(redirect_uri, None, Some(matrix_sdk::authentication::oauth::ClientRegistrationData::new(raw_metadata.clone())), None)
        .build()
        .await?;

    let url = authorization.url.to_string();
    let Some(tachyon_client) = state.get_clients(token) else {
        return Err(anyhow::anyhow!("no client for this session token"));
    };
    tachyon_client.set_pending_oauth(PendingOAuth {
        state: authorization.state.secret().to_owned(),
        client,
    });
    Ok(url)
}

fn oauth_registration_metadata(http_port: u32) -> Result<(matrix_sdk::ruma::serde::Raw<matrix_sdk::authentication::oauth::registration::ClientMetadata>, reqwest::Url), anyhow::Error> {
    use matrix_sdk::authentication::oauth::registration::{ApplicationType, ClientMetadata, Localized, OAuthGrantType};
    use std::iter;

    let redirect_uri = reqwest::Url::parse(&format!("http://127.0.0.1:{}/oauth/callback", http_port))?;

    let registered_uris = {
        let mut registered = redirect_uri.clone();
        // a loopback registration must not pin the port
        let _ = registered.set_port(None);
        vec![OAuthGrantType::AuthorizationCode { redirect_uris: vec![registered] }]
    };

    let mut metadata = ClientMetadata::new(
        ApplicationType::Native,
        registered_uris,
        Localized::new(reqwest::Url::parse("https://tachyon.chat")?, iter::empty()),
    );
    metadata.client_name = Some(Localized::new("Tachyon".to_string(), iter::empty()));

    let raw = matrix_sdk::ruma::serde::Raw::new(&metadata)?;
    Ok((raw, redirect_uri))
}

fn prefill(state: &GlobalState, token: &str, query: &Params) -> (String, String) {
    if let Some(client) = state.get_clients(token) {
        if let Some((username, homeserver)) = client.own_user().get_email_address().as_str().split_once('@') {
            return (username.to_string(), homeserver.to_string());
        }
    }
    (
        query.get("username").cloned().unwrap_or_default(),
        query.get("homeserver").cloned().unwrap_or_default(),
    )
}

// no outer <form>: the swapped action form nested inside one is invalid HTML that IE6 mangles
fn login_form(token: &str, username: &str, homeserver: &str, error: Option<(&str, &str)>, auth: Markup) -> Markup {
    html! {
        div class="container" {
            div class="signin" {
                h2 { (TITLE) }
                p { "Choose the homeserver to connect Tachyon to, then sign in." }
                @if let Some((message, tooltip)) = error {
                    p class="error" style="color: #b30000;" title=(tooltip) {
                        (message)
                    }
                }
                label for="username" { "Username" }
                input type="text" name="username" id="username" placeholder="Username" value=(username);
                label for="homeserver_url" { "Homeserver URL" }
                input type="text" name="homeserver_url" id="homeserver_url"
                    placeholder="https://example.org" value=(homeserver)
                    ic-get-from=(check_url(token)) ic-trigger-on="keyup delayed:500ms"
                    ic-include="input#homeserver_url" ic-target="#auth-area";
                div id="auth-area" {
                    (auth)
                }
            }
        }
    }
}

fn auth_area(token: &str, checked: Option<&ResolvedHomeserver>) -> Markup {
    match checked {
        Some(resolved) => {
            html! {
                span class="check" style="color: #0a7a0a;" { "✔ valid homeserver" }
                (login_action(token, resolved.supports_oauth))
            }
        }
        None => login_action(token, false),
    }
}

fn auth_area_error(token: &str, error: Option<&ResolutionError>) -> Markup {
    match error {
        Some(error) => {
            html! {
                span class="check"
                    title=(error.hint.as_deref().unwrap_or(error.message.as_str()))
                {
                    "✘ " (error.message)
                }
                (login_action(token, false))
            }
        }
        None => login_action(token, false),
    }
}

fn login_action(token: &str, sso: bool) -> Markup {
    html! {
        form method="POST" action=(page_url(token)) ic-post-to=(page_url(token))
            ic-target=".content" ic-include="#username,#homeserver_url"
        {
            @if sso {
                input type="hidden" name="action" value="sso";
                button type="submit" class="btn btn-primary" {
                    span class="btn-shine" {}
                    span class="btn-label" { "Login using SSO" }
                }
            } @else {
                label for="password" { "Password" }
                input type="password" name="password" id="password" placeholder="Password";
                br;
                button type="submit" class="btn btn-primary" {
                    span class="btn-shine" {}
                    span class="btn-label" { "Log In" }
                }
            }
        }
    }
}

fn page_url(token: &str) -> String {
    format!("/tachyon/login?t={}", token)
}

fn check_url(token: &str) -> String {
    format!("/tachyon/login/check?t={}", token)
}

fn login_error_page(token: &str, username: &str, homeserver: &str, error: &ResolutionError) -> Response {
    let tooltip = error.hint.as_deref().unwrap_or(error.message.as_str());
    login_error_response(token, username, homeserver, Some((error.message.as_str(), tooltip)))
}

fn login_error_message(token: &str, username: &str, homeserver: &str, message: &str) -> Response {
    login_error_response(token, username, homeserver, Some((message, message)))
}

fn login_error_response(token: &str, username: &str, homeserver: &str, error: Option<(&str, &str)>) -> Response {
    let auth = login_action(token, false);
    Html(
        layout::tachyon_page_no_nav(login_form(token, username, homeserver, error, auth)).into_string(),
    )
        .into_response()
}

pub(crate) fn redirect(target: &str, headers: &HeaderMap) -> Response {
    let is_ic_request = headers
        .get("X-IC-Request")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    if is_ic_request {
        Response::builder()
            .status(StatusCode::OK)
            .header("X-IC-Redirect", target)
            .body(Body::empty())
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(LOCATION, target)
            .body(Body::empty())
            .unwrap()
    }
}

// the driver toast still fires; whichever path the user follows first wins
pub(crate) async fn redirect_to_next_step(state: &GlobalState, token: &str, headers: &HeaderMap) -> Response {
    let Some(tachyon_client) = state.get_clients(token) else {
        return error_page("Session lost — reopen Tachyon from the Messenger alert.");
    };

    let cross_signed = check_device_is_crossed_signed(&tachyon_client.matrix_client_ready()).await.unwrap_or(false);
    if cross_signed {
        redirect(&format!("/tachyon?t={}", token), headers)
    } else {
        redirect(&format!("/tachyon/confirm_device?t={}", token), headers)
    }
}

pub(crate) fn error_page(message: &str) -> Response {
    Html(layout::tachyon_page_no_nav(html! {
        div class="container" {
            h2 { "Something went wrong" }
            p style="color: #b30000;" { (message) }
        }
    }).into_string())
        .into_response()
}
