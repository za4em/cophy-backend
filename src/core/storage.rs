use std::time::Duration;

use aws_config::BehaviorVersion;
use aws_sdk_s3::{
    config::{Credentials, Region, RequestChecksumCalculation, ResponseChecksumValidation},
    presigning::PresigningConfig,
};

use super::{
    config::StorageConfig,
    errors::{AppError, StorageFault},
};

pub(crate) struct StoredObject {
    pub(crate) content_type: Option<String>,
}

#[derive(Clone)]
pub(crate) struct Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
    upload_ttl: Duration,
    read_ttl: Duration,
}

impl Storage {
    pub(crate) async fn connect(config: StorageConfig) -> Result<Self, AppError> {
        let shared = aws_config::defaults(BehaviorVersion::latest())
            .region(Region::new("auto"))
            .credentials_provider(Credentials::new(
                config.access_key_id,
                config.secret_access_key,
                None,
                None,
                "R2",
            ))
            .load()
            .await;
        let sdk = aws_sdk_s3::config::Builder::from(&shared)
            .endpoint_url(format!(
                "https://{}.r2.cloudflarestorage.com",
                config.account_id
            ))
            .request_checksum_calculation(RequestChecksumCalculation::WhenRequired)
            .response_checksum_validation(ResponseChecksumValidation::WhenRequired)
            .build();
        let client = aws_sdk_s3::Client::from_conf(sdk);
        let storage = Self {
            client,
            bucket: config.bucket,
            upload_ttl: config.upload_ttl,
            read_ttl: config.read_ttl,
        };
        storage.check_connection().await?;
        Ok(storage)
    }

    async fn check_connection(&self) -> Result<(), AppError> {
        self.client
            .head_bucket()
            .bucket(&self.bucket)
            .send()
            .await
            .map_err(|err| {
                tracing::error!(error = %err, bucket = %self.bucket, "r2 bucket is not reachable");
                AppError::Storage(StorageFault::Unreachable {
                    bucket: self.bucket.clone(),
                })
            })?;
        Ok(())
    }

    pub(crate) async fn presign_put(&self, key: &str) -> Result<String, AppError> {
        let presigned = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presign_config(self.upload_ttl)?)
            .await
            .map_err(|err| {
                storage_error(StorageFault::UploadUrl, "creating the upload URL", err)
            })?;
        Ok(presigned.uri().to_string())
    }

    pub(crate) async fn presign_get(&self, key: &str) -> Result<String, AppError> {
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presign_config(self.read_ttl)?)
            .await
            .map_err(|err| {
                storage_error(
                    StorageFault::ReadUrl,
                    "creating a read URL for the image",
                    err,
                )
            })?;
        Ok(presigned.uri().to_string())
    }

    pub(crate) async fn head(&self, key: &str) -> Result<Option<StoredObject>, AppError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(object) => Ok(Some(StoredObject {
                content_type: object.content_type().map(str::to_owned),
            })),
            Err(err) if err.as_service_error().is_some_and(|err| err.is_not_found()) => Ok(None),
            Err(err) => Err(storage_error(
                StorageFault::Head,
                "checking whether the uploaded image exists",
                err,
            )),
        }
    }

    pub(crate) async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|err| storage_error(StorageFault::DeleteObject, "deleting the image", err))?;
        Ok(())
    }
}

fn presign_config(ttl: Duration) -> Result<PresigningConfig, AppError> {
    PresigningConfig::expires_in(ttl).map_err(|err| {
        tracing::error!(error = %err, "r2 signature lifetime is invalid");
        AppError::Storage(StorageFault::BadTtl)
    })
}

fn storage_error(fault: StorageFault, action: &str, err: impl std::fmt::Display) -> AppError {
    tracing::error!(error = %err, "r2 failed while {action}");
    AppError::Storage(fault)
}
