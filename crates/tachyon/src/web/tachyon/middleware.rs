use axum::extract::State;
use axum::http::{Request, Response, StatusCode};
use axum::body::Body;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::http::header::{CONTENT_TYPE, COOKIE, LOCATION, SET_COOKIE};
use http_body_util::BodyExt;
use log::error;
use maud::html;
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::{layout, Params};

pub async fn is_authenticated(
    State(state): State<GlobalState>,
    req: Request<Body>,
    next: Next,
) -> impl IntoResponse {
    if let Some(token) = req.extensions().get::<String>() {
        if state.tachyon_clients().get(token).is_some() {
            return next.run(req).await;
        }
    }

    Response::builder()
        .status(StatusCode::TEMPORARY_REDIRECT)
        .header(SET_COOKIE, clear_token_cookie())
        .header(LOCATION, "/tachyon/login" )
        .body(Body::empty())
        .unwrap()
}

fn clear_token_cookie() -> &'static str {
    "t=; Max-Age=-1; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; SameSite=Lax;"
}

pub fn set_token_cookie(token: &str) -> String {
    format!("t={}; SameSite=Lax; Path=/", token)
}

pub async fn extract_token(req: Request<Body>, next: Next) -> impl IntoResponse {
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

pub async fn intercooler_layout_wrapper(req: Request<Body>, next: Next) -> Response<Body> {

    let is_ic_request = req
        .headers()
        .get("X-IC-Request")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let response = next.run(req).await;

    let is_html = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.starts_with("text/html"))
        .unwrap_or(false);

    if is_ic_request || !is_html {
        return response;
    }

    let (parts, body) = response.into_parts();

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            error!("Could not read body of HTML response: {:?}", e);
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .unwrap();
        }
    };

    let fragment = match std::str::from_utf8(&bytes) {
        Ok(s) => s,
        Err(_) => {
            let mut response = Response::from_parts(parts, Body::from(bytes));
            return response;
        }
    };

    let wrapped = layout::tachyon_page(html! { (maud::PreEscaped(fragment)) }).into_string();

    let mut response = Response::from_parts(parts, Body::from(wrapped));
    response.headers_mut().remove(axum::http::header::CONTENT_LENGTH);

    response
}
