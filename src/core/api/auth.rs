use axum::{Json, extract::FromRequestParts, http::request::Parts};
use serde::Serialize;

pub(super) const MOCK_USER_ID: &str = "mock-user";

pub(super) struct CurrentUser {
    pub(super) id: String,
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(CurrentUser {
            id: MOCK_USER_ID.to_owned(),
        })
    }
}

#[derive(Serialize)]
pub(super) struct AuthSession {
    token: &'static str,
    user_id: &'static str,
}

pub(super) async fn post_apple() -> Json<AuthSession> {
    Json(mock_session())
}

fn mock_session() -> AuthSession {
    AuthSession {
        token: "mock",
        user_id: MOCK_USER_ID,
    }
}
