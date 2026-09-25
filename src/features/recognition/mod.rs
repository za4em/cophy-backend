mod reader;

use crate::core::{errors::AppError, models::Bean};
use crate::features::{
    beans::Beans,
    uploads::{ClaimedUpload, Uploads},
};

pub(crate) use reader::BagReader;

pub(crate) struct RecognizeResult {
    pub(crate) bean: Bean,
    pub(crate) created: bool,
}

/// Turns an uploaded bag photo into a bean.
#[derive(Clone)]
pub(crate) struct Recognition {
    uploads: Uploads,
    reader: BagReader,
    beans: Beans,
}

impl Recognition {
    pub(crate) fn new(uploads: Uploads, reader: BagReader, beans: Beans) -> Self {
        Self {
            uploads,
            reader,
            beans,
        }
    }

    /// Claims the upload, reads the bag, and saves the bean.
    /// A failed read puts the ticket back so the client can retry without uploading again.
    /// A retry after a finished recognition returns the bean that was already saved.
    pub(crate) async fn recognize(
        &self,
        user_id: &str,
        id: &str,
    ) -> Result<RecognizeResult, AppError> {
        let key = self.uploads.storage_key(id)?;
        if let Some(result) = self.saved_bean(user_id, &key).await? {
            return Ok(result);
        }

        let upload = match self.uploads.claim(user_id, id).await {
            Ok(upload) => upload,
            Err(AppError::UploadNotFound) => {
                return self
                    .saved_bean(user_id, &key)
                    .await?
                    .ok_or(AppError::UploadNotFound);
            }
            Err(err) => return Err(err),
        };

        let bean = match self.read_and_save(user_id, &upload).await {
            Ok(bean) => bean,
            Err(err) => {
                self.restore_ticket(&upload).await;
                return Err(err);
            }
        };
        self.finish_ticket(&upload.key).await;
        Ok(RecognizeResult {
            bean,
            created: true,
        })
    }

    async fn saved_bean(
        &self,
        user_id: &str,
        key: &str,
    ) -> Result<Option<RecognizeResult>, AppError> {
        let Some(bean) = self.beans.find_by_image_key(user_id, key).await? else {
            return Ok(None);
        };
        self.finish_ticket(key).await;
        Ok(Some(RecognizeResult {
            bean,
            created: false,
        }))
    }

    async fn finish_ticket(&self, key: &str) {
        if let Err(err) = self.uploads.finish(key).await {
            tracing::error!(
                %key,
                error = ?err,
                "bean was saved but the upload ticket remains"
            );
        }
    }

    async fn restore_ticket(&self, upload: &ClaimedUpload) {
        if let Err(err) = self.uploads.release(upload).await {
            tracing::error!(
                key = %upload.key,
                error = ?err,
                "could not restore the upload ticket"
            );
        }
    }

    async fn read_and_save(&self, user_id: &str, upload: &ClaimedUpload) -> Result<Bean, AppError> {
        let image_url = self.uploads.read_url(&upload.key).await?;
        let draft = self.reader.read(&image_url).await?;
        self.beans.save(user_id, draft, &upload.key).await
    }
}
