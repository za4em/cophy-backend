mod core;
mod features;

pub use core::app_state::AppState;
pub use core::config::Config;

pub fn router(state: AppState) -> axum::Router {
    core::api::router(state)
}
