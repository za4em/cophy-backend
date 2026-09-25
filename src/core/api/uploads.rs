use axum::{Json, Router, extract::State, routing::post};

use super::auth::CurrentUser;
use crate::{
    core::{app_state::AppState, errors::AppError},
    features::uploads::UploadTicket,
};

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/uploads", post(create_upload))
}

async fn create_upload(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<UploadTicket>, AppError> {
    Ok(Json(state.uploads.issue(&user.id).await?))
}
