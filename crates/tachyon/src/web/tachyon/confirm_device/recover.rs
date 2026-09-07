use axum::response::Html;
use crate::web::tachyon::layout::error_fragment;

pub async fn get_recover() -> Html<String> {
    Html(error_fragment("Device confirmation is being rewired.").into_string())
}

pub async fn post_recover() -> Html<String> {
    Html(error_fragment("Device confirmation is being rewired.").into_string())
}
