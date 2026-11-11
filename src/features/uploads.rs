pub(crate) mod api;
pub(crate) mod model;
mod repository;

use std::time::Duration;

use sqlx::SqlitePool;
use uuid::Uuid;

use crate::core::{config::UploadsConfig, errors::AppError, storage::Storage};

pub(crate) use model::{ClaimedUpload, UploadTicket};

#[derive(Clone)]
pub(crate) struct Uploads {
    pool: SqlitePool,
    storage: Storage,
    key_prefix: String,
    abandon_after: Duration,
}

impl Uploads {
    pub(crate) fn new(pool: SqlitePool, storage: Storage, config: UploadsConfig) -> Self {
        Self {
            pool,
            storage,
            key_prefix: config.key_prefix,
            abandon_after: config.abandon_after,
        }
    }

    pub(crate) fn storage_key(&self, id: &str) -> Result<String, AppError> {
        storage_key(&self.key_prefix, id)
    }

    pub(crate) async fn issue(&self, user_id: &str) -> Result<UploadTicket, AppError> {
        let id = Uuid::new_v4();
        let key = format!("{}/{id}", self.key_prefix);
        repository::insert(&self.pool, user_id, &key).await?;

        let upload_url = match self.storage.presign_put(&key).await {
            Ok(url) => url,
            Err(err) => {
                if let Err(delete_err) = repository::delete(&self.pool, &key).await {
                    tracing::error!(
                        %key,
                        error = ?delete_err,
                        "presign failed and the upload ticket remains"
                    );
                }
                return Err(err);
            }
        };

        Ok(UploadTicket {
            id: id.to_string(),
            upload_url,
        })
    }

    pub(crate) async fn claim(&self, user_id: &str, id: &str) -> Result<ClaimedUpload, AppError> {
        let key = self.storage_key(id)?;
        repository::claim(&self.pool, user_id, &key).await
    }

    pub(crate) async fn release(&self, upload: &ClaimedUpload) -> Result<(), AppError> {
        repository::release(&self.pool, &upload.key).await
    }

    pub(crate) async fn release_processing(&self) -> Result<(), AppError> {
        repository::release_processing(&self.pool).await
    }

    pub(crate) async fn finish(&self, key: &str) -> Result<(), AppError> {
        repository::delete(&self.pool, key).await
    }

    pub(crate) async fn read_url(&self, key: &str) -> Result<String, AppError> {
        let Some(object) = self.storage.head(key).await? else {
            return Err(AppError::ImageMissing);
        };
        normalize_content_type(object.content_type.as_deref().unwrap_or(""))?;
        self.storage.presign_get(key).await
    }

    pub(crate) async fn expire_abandoned(&self) -> Result<(), AppError> {
        let keys = repository::delete_abandoned(&self.pool, self.abandon_after).await?;
        for key in keys {
            if let Err(err) = self.storage.delete(&key).await {
                tracing::error!(%key, error = ?err, "abandoned upload remains in s3");
            }
        }
        Ok(())
    }
}

fn storage_key(prefix: &str, id: &str) -> Result<String, AppError> {
    let uuid = Uuid::parse_str(id.trim()).map_err(|_| AppError::UploadIdRequired)?;
    Ok(format!("{prefix}/{uuid}"))
}

fn normalize_content_type(value: &str) -> Result<&'static str, AppError> {
    match value.split(';').next().unwrap_or("").trim() {
        "image/jpeg" | "image/jpg" => Ok("image/jpeg"),
        "image/png" => Ok("image/png"),
        "image/webp" => Ok("image/webp"),
        "image/heic" | "image/heif" => Ok("image/heic"),
        _ => Err(AppError::UnsupportedImageType),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_image_types() {
        assert_eq!(normalize_content_type("image/jpg").unwrap(), "image/jpeg");
        assert_eq!(
            normalize_content_type("image/png; charset=binary").unwrap(),
            "image/png"
        );
        assert_eq!(normalize_content_type("image/heif").unwrap(), "image/heic");
    }

    #[test]
    fn rejects_a_non_image() {
        assert_eq!(
            normalize_content_type("text/plain"),
            Err(AppError::UnsupportedImageType)
        );
    }

    #[test]
    fn storage_key_hides_the_prefix() {
        let id = "11111111-1111-4111-8111-111111111111";
        assert_eq!(storage_key("uploads", id).unwrap(), format!("uploads/{id}"));
        assert_eq!(
            storage_key("uploads", "not-a-uuid"),
            Err(AppError::UploadIdRequired)
        );
    }
}
