use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};

use super::{auth::CurrentUser, pagination::ListQuery};
use crate::core::{
    app_state::AppState,
    errors::AppError,
    models::{Bean, BeanDraft},
};

pub(super) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/beans", get(list).post(create))
        .route("/api/beans/{id}", get(get_one).put(update).delete(remove))
}

async fn list(
    State(state): State<AppState>,
    user: CurrentUser,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Bean>>, AppError> {
    Ok(Json(
        state
            .beans
            .list(
                &user.id,
                query.before(),
                query.limit(state.beans.page_size()),
            )
            .await?,
    ))
}

async fn get_one(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<Bean>, AppError> {
    Ok(Json(state.beans.get(&user.id, id).await?))
}

async fn create(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(draft): Json<BeanDraft>,
) -> Result<(StatusCode, Json<Bean>), AppError> {
    let bean = state.beans.create(&user.id, draft).await?;
    Ok((StatusCode::CREATED, Json(bean)))
}

async fn update(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(draft): Json<BeanDraft>,
) -> Result<Json<Bean>, AppError> {
    Ok(Json(state.beans.update(&user.id, id, draft).await?))
}

async fn remove(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    state.beans.delete(&user.id, id).await?;
    Ok(StatusCode::OK)
}
