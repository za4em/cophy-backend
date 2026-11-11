use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use serde::Deserialize;

use super::Recognition;
use crate::core::api::CurrentUser;
use crate::core::errors::AppError;
use crate::features::beans::model::Bean;

pub(crate) fn routes() -> Router<Recognition> {
    Router::new().route("/api/beans/recognize", post(recognize))
}

#[derive(Deserialize)]
struct RecognizeRequest {
    id: String,
}

async fn recognize(
    State(recognition): State<Recognition>,
    user: CurrentUser,
    Json(request): Json<RecognizeRequest>,
) -> Result<(StatusCode, Json<Bean>), AppError> {
    let result = recognition.recognize(&user.id, &request.id).await?;
    let status = if result.created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(result.bean)))
}
