pub(crate) mod api;
pub(crate) mod model;
mod repository;

use futures::future::join_all;

use model::{Bean, BeanDraft, decode_notes};
use sqlx::SqlitePool;

use crate::core::{config::BeansConfig, errors::AppError, storage::Storage};

#[derive(Clone)]
pub(crate) struct Beans {
    pool: SqlitePool,
    storage: Storage,
    page_size: i64,
}

impl Beans {
    pub(crate) fn new(pool: SqlitePool, storage: Storage, config: BeansConfig) -> Self {
        Self {
            pool,
            storage,
            page_size: config.page_size,
        }
    }

    pub(crate) fn page_size(&self) -> i64 {
        self.page_size
    }

    pub(crate) async fn list(
        &self,
        user_id: &str,
        before: Option<i64>,
        limit: i64,
    ) -> Result<Vec<Bean>, AppError> {
        let rows = repository::list(&self.pool, user_id, before, limit).await?;
        Ok(join_all(rows.into_iter().map(|row| {
            let storage = self.storage.clone();
            async move { listed(storage, row).await }
        }))
        .await)
    }

    pub(crate) async fn get(&self, user_id: &str, id: i64) -> Result<Bean, AppError> {
        let row = repository::fetch(&self.pool, user_id, id).await?;
        self.bean(&row).await
    }

    pub(crate) async fn create(&self, user_id: &str, draft: BeanDraft) -> Result<Bean, AppError> {
        let draft = draft.require_name()?;
        let row = repository::insert(&self.pool, user_id, &draft, None).await?;
        self.bean(&row).await
    }

    pub(crate) async fn update(
        &self,
        user_id: &str,
        id: i64,
        draft: BeanDraft,
    ) -> Result<Bean, AppError> {
        let draft = draft.require_name()?;
        let row = repository::update(&self.pool, user_id, id, &draft).await?;
        self.bean(&row).await
    }

    pub(crate) async fn delete(&self, user_id: &str, id: i64) -> Result<(), AppError> {
        let image_key = repository::delete(&self.pool, user_id, id).await?;
        if let Some(key) = image_key
            && let Err(err) = self.storage.delete(&key).await
        {
            tracing::error!(%key, error = %err, "bean was deleted but its image remains in s3");
        }
        Ok(())
    }

    pub(crate) async fn save(
        &self,
        user_id: &str,
        draft: BeanDraft,
        image_key: &str,
    ) -> Result<Bean, AppError> {
        let row = repository::insert(&self.pool, user_id, &draft, Some(image_key)).await?;
        self.bean(&row).await
    }

    pub(crate) async fn find_by_image_key(
        &self,
        user_id: &str,
        image_key: &str,
    ) -> Result<Option<Bean>, AppError> {
        match repository::find_by_image_key(&self.pool, user_id, image_key).await? {
            Some(row) => Ok(Some(self.bean(&row).await?)),
            None => Ok(None),
        }
    }

    async fn bean(&self, row: &repository::BeanRow) -> Result<Bean, AppError> {
        Ok(Bean {
            id: row.id,
            name: row.name.clone(),
            origin: row.origin.clone(),
            roaster: row.roaster.clone(),
            process: row.process.clone(),
            elevation: row.elevation.clone(),
            tasting_notes: decode_notes(&row.tasting_notes),
            image_url: self.image_url(row).await?,
        })
    }

    async fn image_url(&self, row: &repository::BeanRow) -> Result<Option<String>, AppError> {
        match &row.image_key {
            Some(key) => Ok(Some(self.storage.presign_get(key).await?)),
            None => Ok(None),
        }
    }
}

async fn listed(storage: Storage, row: repository::BeanRow) -> Bean {
    let image_url = match &row.image_key {
        Some(key) => match storage.presign_get(key).await {
            Ok(url) => Some(url),
            Err(err) => {
                tracing::error!(%key, error = %err, "could not sign an image url for the list");
                None
            }
        },
        None => None,
    };
    Bean {
        id: row.id,
        name: row.name,
        origin: row.origin,
        roaster: row.roaster,
        process: row.process,
        elevation: row.elevation,
        tasting_notes: decode_notes(&row.tasting_notes),
        image_url,
    }
}
