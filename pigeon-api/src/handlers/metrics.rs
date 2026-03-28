use axum::extract::State;
use axum::http::header;
use axum::response::IntoResponse;

use crate::state::AppState;

pub async fn render(State(state): State<AppState>) -> impl IntoResponse {
    let body = (state.metrics_render)();
    ([(header::CONTENT_TYPE, "text/plain; version=0.0.4")], body)
}
