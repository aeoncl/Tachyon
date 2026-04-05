mod middleware;
mod layout;
mod matrix_auth;
mod login;
mod verify_device;

use crate::tachyon::global_state::GlobalState;
use axum::body::Body;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::http::{Response, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use lazy_static_include::lazy_static_include_bytes;
use maud::html;
use std::str::FromStr;
use crate::tachyon::repository::RepositoryStr;

lazy_static_include_bytes! {
    INDEX => "./assets/web/tachyon/index.html",
    FAVICON => "./assets/web/tachyon/favicon.ico",
    STYLE => "./assets/web/tachyon/style.css",
    LOGO => "./assets/web/tachyon/tachyon_logo.png",
    LOGO_2 => "./assets/web/tachyon/tachyon_logo_2.png",
    INTERCOOLER => "./assets/web/tachyon/intercooler-1.2.4.min.js",
    JQUERY => "./assets/web/tachyon/jquery-1.10.0.min.js",
    SHIELD_VERIFY => "./assets/web/tachyon/shield_verify.png",
    VERIFY_SCRIPT => "./assets/web/tachyon/verify.js",
    TREMOVE_SCRIPT => "./assets/web/tachyon/tremove.js",
}


pub fn tachyon_router(state: GlobalState) -> Router<GlobalState> {
    Router::new()
        //Secured v
        .route("/test", get(serve_index))
        .route("/verify_device", get(verify_device::get_verify))
        .route("/verify_device/reset-identity", post(verify_device::post_reset_identity))
        .route("/verify_device/restore", get(verify_device::get_restore))
        .route("/verify_device/restore", post(verify_device::post_restore))
        .route("/login/nfy", get(login::get_login_nfy))
        .layer(from_fn_with_state(state.clone(), middleware::is_authenticated))
        //Unsecured v
        .route("/", get(serve_index))
        .route("/login", get(login::get_login_page))
        .route("/login/request", get(login::get_login_request))
        .route("/login/request", post(login::post_login_request))
        .route("/auth", get(matrix_auth::get_auth))
        .route("/auth", post(matrix_auth::post_auth))
        .route("/{file}", get(serve_static))
        .layer(from_fn(middleware::extract_token))
        .with_state(state)
}

type Params = std::collections::HashMap<String, String>;

async fn serve_index() -> Html<String> {
    Html(
        layout::tachyon_page(html! {
            h2 { "Tachyon is running..." }
        })
            .into_string(),
    )
}

async fn serve_static(Path(file): Path<String>) -> Response<Body> {
    let (data, content_type) = match file.as_str() {
        "favicon.ico" => (*FAVICON, "image/x-icon"),
        "style.css" => (*STYLE, "text/css"),
        "tachyon_logo.png" => (*LOGO, "image/png"),
        "tachyon_logo_2.png" => (*LOGO_2, "image/png"),
        "shield_verify.png" => (*SHIELD_VERIFY, "image/png"),
        "intercooler-1.2.4.min.js" => (*INTERCOOLER, "text/javascript"),
        "jquery-1.10.0.min.js" => (*JQUERY, "text/javascript"),
        "verify.js" => (*VERIFY_SCRIPT, "text/javascript"),
        "tremove.js" => (*TREMOVE_SCRIPT, "text/javascript"),
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