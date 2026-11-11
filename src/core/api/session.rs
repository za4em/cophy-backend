use axum::extract::FromRequestParts;
use axum::http::request::Parts;

pub(crate) const MOCK_USER_ID: &str = "mock-user";

pub(crate) struct CurrentUser {
    pub(crate) id: String,
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
