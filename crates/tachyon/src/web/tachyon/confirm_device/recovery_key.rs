use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::tachyon::global_state::GlobalState;
use crate::web::tachyon::login::{error_page, redirect_to_next_step};

pub async fn post_saved(
    State(state): State<GlobalState>,
    Extension(token): Extension<String>,
    headers: HeaderMap,
) -> Response {
    if state.get_clients(&token).is_none() {
        return error_page("No session for this token.");
    }

    redirect_to_next_step(&state, &token, &headers).await
}
