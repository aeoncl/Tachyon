use axum::response::Html;
use crate::web::tachyon::layout::error_fragment;

pub(super) mod recover;
pub(super) mod reset_identity;
pub(super) mod other_device;

pub async fn get_confirm() -> Html<String> {
    Html(error_fragment("Device confirmation is being rewired.").into_string())
}
