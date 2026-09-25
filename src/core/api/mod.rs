mod auth;
mod beans;
mod error;
mod pagination;
mod recognition;
mod uploads;

use axum::Router;

use super::app_state::AppState;

pub(crate) fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/auth/apple", axum::routing::post(auth::post_apple))
        .merge(beans::routes())
        .merge(uploads::routes())
        .merge(recognition::routes())
        .with_state(state)
}
