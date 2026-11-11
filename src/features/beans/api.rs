use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
};

use serde::Deserialize;

use super::Beans;
use super::model::{Bean, BeanDraft};
use crate::core::api::CurrentUser;
use crate::core::errors::AppError;

pub(crate) fn routes() -> Router<Beans> {
    Router::new()
        .route("/api/beans", get(list).post(create))
        .route("/api/beans/{id}", get(get_one).put(update).delete(remove))
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct ListQuery {
    limit: Option<i64>,
    before: Option<i64>,
}

impl ListQuery {
    fn limit(&self, max: i64) -> i64 {
        match self.limit {
            Some(limit) if limit > 0 => limit.min(max),
            _ => max,
        }
    }

    fn before(&self) -> Option<i64> {
        self.before
    }
}

async fn list(
    State(beans): State<Beans>,
    user: CurrentUser,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<Bean>>, AppError> {
    Ok(Json(
        beans
            .list(&user.id, query.before(), query.limit(beans.page_size()))
            .await?,
    ))
}

async fn get_one(
    State(beans): State<Beans>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<Json<Bean>, AppError> {
    Ok(Json(beans.get(&user.id, id).await?))
}

async fn create(
    State(beans): State<Beans>,
    user: CurrentUser,
    Json(draft): Json<BeanDraft>,
) -> Result<(StatusCode, Json<Bean>), AppError> {
    let bean = beans.create(&user.id, draft).await?;
    Ok((StatusCode::CREATED, Json(bean)))
}

async fn update(
    State(beans): State<Beans>,
    user: CurrentUser,
    Path(id): Path<i64>,
    Json(draft): Json<BeanDraft>,
) -> Result<Json<Bean>, AppError> {
    Ok(Json(beans.update(&user.id, id, draft).await?))
}

async fn remove(
    State(beans): State<Beans>,
    user: CurrentUser,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    beans.delete(&user.id, id).await?;
    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_caps_and_defaults() {
        assert_eq!(ListQuery::default().limit(100), 100);
        assert_eq!(query(Some(0)).limit(100), 100);
        assert_eq!(query(Some(-1)).limit(100), 100);
        assert_eq!(query(Some(20)).limit(100), 20);
        assert_eq!(query(Some(500)).limit(100), 100);
    }

    fn query(limit: Option<i64>) -> ListQuery {
        ListQuery {
            limit,
            before: None,
        }
    }
}
