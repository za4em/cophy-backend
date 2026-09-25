use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;

use super::auth::CurrentUser;
use crate::core::{app_state::AppState, errors::AppError, models::Bean};

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/beans/recognize", post(recognize))
}

#[derive(Deserialize)]
struct RecognizeRequest {
    id: String,
}

async fn recognize(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(request): Json<RecognizeRequest>,
) -> Result<(StatusCode, Json<Bean>), AppError> {
    let result = state.recognition.recognize(&user.id, &request.id).await?;
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(result.bean)))
}
