use axum::{Json, Router, extract::State, routing::post};

use super::{UploadTicket, Uploads};
use crate::core::api::CurrentUser;
use crate::core::errors::AppError;

pub(crate) fn routes() -> Router<Uploads> {
    Router::new().route("/api/uploads", post(create_upload))
}

async fn create_upload(
    State(uploads): State<Uploads>,
    user: CurrentUser,
) -> Result<Json<UploadTicket>, AppError> {
    Ok(Json(uploads.issue(&user.id).await?))
}
