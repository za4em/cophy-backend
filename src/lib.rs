mod app_state;
mod core;
mod features;

pub use app_state::AppState;
pub use core::config::Config;

pub fn router(state: AppState) -> axum::Router {
    axum::Router::new()
        .merge(features::auth::api::routes())
        .merge(features::beans::api::routes().with_state(state.beans))
        .merge(features::uploads::api::routes().with_state(state.uploads))
        .merge(features::recognition::api::routes().with_state(state.recognition))
}
