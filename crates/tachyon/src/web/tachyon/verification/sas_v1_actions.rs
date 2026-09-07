use axum::response::Html;
use crate::web::tachyon::layout::error_fragment;

pub(crate) async fn post_sas_v1_action() -> Html<String> {
    Html(error_fragment("Device verification is being rewired.").into_string())
}
