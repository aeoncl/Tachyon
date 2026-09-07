pub(super) mod sas_v1_actions;

use axum::response::Html;
use crate::web::tachyon::layout::error_fragment;

pub async fn get_verification_poll() -> Html<String> {
    Html(error_fragment("Device verification is being rewired.").into_string())
}
