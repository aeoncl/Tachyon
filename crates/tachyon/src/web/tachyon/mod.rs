mod middleware;
mod layout;
mod matrix_auth;
mod login;
mod confirm_device;
mod verification;

use crate::tachyon::global_state::GlobalState;
use axum::body::Body;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, Method, Response, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, head, post};
use axum::Router;
use lazy_static_include::lazy_static_include_bytes;
use maud::html;
use std::str::FromStr;
use sha1::{Digest, Sha1};
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::confirm_device::{reset_identity, recover, other_device};

lazy_static_include_bytes! {
    INDEX => "./assets/web/tachyon/index.html",
    FAVICON => "./assets/web/tachyon/favicon.ico",
    STYLE => "./assets/web/tachyon/style.css",
    INTERCOOLER => "./assets/web/tachyon/intercooler-1.2.4.min.js",
    JQUERY => "./assets/web/tachyon/jquery-1.10.0.min.js",
    VERIFY_SCRIPT => "./assets/web/tachyon/verify.js",
    TREMOVE_SCRIPT => "./assets/web/tachyon/tremove.js",
    SHIELD_VERIFY => "./assets/web/tachyon/img/shield_verify.png",
    LOGO => "./assets/web/tachyon/img/tachyon_logo.png",
    LOGO_2 => "./assets/web/tachyon/img/tachyon_logo_2.png",
    SMILEY_SCARED => "./assets/web/tachyon/img/scared-emoticon.gif",
    SMILEY_BOSSEORDI => "./assets/web/tachyon/img/smiley_bosseordi.gif",
    SMILEY_LOOKING => "./assets/web/tachyon/img/text-looking.gif",
}


pub fn tachyon_router(state: GlobalState) -> Router<GlobalState> {
    Router::new()
        //Secured v
        .route("/test", get(serve_index))
        .route("/confirm_device", get(confirm_device::get_confirm))
        .route("/confirm_device/reset_identity", post(reset_identity::post_reset_identity))
        .route("/confirm_device/reset_identity", get(reset_identity::get_reset_identity))
        .route("/confirm_device/recover", get(recover::get_recover))
        .route("/confirm_device/recover", post(recover::post_recover))
        .route("/confirm_device/other_device", get(other_device::get_other_device))
        .route("/confirm_device/other_device", post(other_device::post_other_device))
        .route("/verification", get(verification::get_verification))
        .route("/login/nfy", get(login::get_login_nfy))
        .layer(from_fn_with_state(state.clone(), middleware::is_authenticated))
        .layer(from_fn(middleware::intercooler_layout_wrapper))
        //Unsecured v
        .route("/", get(serve_index))
        .route("/login", get(login::get_login_page))
        .route("/login/request", get(login::get_login_request))
        .route("/login/request", post(login::post_login_request))
        .route("/auth", get(matrix_auth::get_auth))
        .route("/auth", post(matrix_auth::post_auth))
        .route("/img/{file}", get(serve_static))
        .route("/img/{file}", head(serve_static))
        .route("/{file}", get(serve_static))
        .route("/{file}", head(serve_static))
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

async fn serve_static(
    method: Method,
    Path(file): Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    let (data, content_type) = match file.as_str() {
        "favicon.ico" => (*FAVICON, "image/x-icon"),
        "style.css" => (*STYLE, "text/css"),
        "intercooler-1.2.4.min.js" => (*INTERCOOLER, "text/javascript"),
        "jquery-1.10.0.min.js" => (*JQUERY, "text/javascript"),
        "verify.js" => (*VERIFY_SCRIPT, "text/javascript"),
        "tremove.js" => (*TREMOVE_SCRIPT, "text/javascript"),
        "tachyon_logo.png" => (*LOGO, "image/png"),
        "tachyon_logo_2.png" => (*LOGO_2, "image/png"),
        "shield_verify.png" => (*SHIELD_VERIFY, "image/png"),
        "scared-emoticon.gif" => (*SMILEY_SCARED, "image/gif"),
        "smiley_bosseordi.gif" => (*SMILEY_BOSSEORDI, "image/gif"),
        "text-looking.gif" => (*SMILEY_LOOKING, "image/gif"),
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap()
        }
    };

    if let Some(if_none_match) = headers.get(IF_NONE_MATCH) {
        if let Ok(if_none_match) = if_none_match.to_str() {
            if sha_1_encode(&data) == if_none_match {
                return Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .body(Body::empty())
                    .unwrap();
            }
        }
    }


    let response_builder = Response::builder()
        .header(CONTENT_TYPE, content_type)
        .header(CACHE_CONTROL, "public, max-age=604800, must-revalidate")
        .header(ETAG, sha_1_encode(&data))
        .status(
            StatusCode::OK,
        );

    if method == Method::GET {
        response_builder.body(Body::from(data)).unwrap()
    } else {
        response_builder.body(Body::empty()).unwrap()
    }
}

fn sha_1_encode(input: &[u8]) -> String {

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, input);
    let result = hasher.finalize();
    hex::encode(result)

}
