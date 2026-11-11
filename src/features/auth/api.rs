use axum::{Json, Router, routing::post};
use serde::Serialize;

use super::AppleSession;

pub(crate) fn routes() -> Router {
    Router::new().route("/api/auth/apple", post(post_apple))
}

#[derive(Serialize)]
struct AuthSession {
    token: &'static str,
    user_id: &'static str,
}

async fn post_apple() -> Json<AuthSession> {
    let AppleSession { token, user_id } = super::apple_login();
    Json(AuthSession { token, user_id })
}
