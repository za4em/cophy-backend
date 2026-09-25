use std::fmt::{self, Display};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::core::errors::{AppError, ReaderFault, StorageFault};

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&client_message(self))
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (status(&self), client_message(&self)).into_response()
    }
}

fn status(err: &AppError) -> StatusCode {
    match err {
        AppError::NameRequired
        | AppError::UploadIdRequired
        | AppError::ImageMissing
        | AppError::UnsupportedImageType => StatusCode::BAD_REQUEST,
        AppError::BeanNotFound { .. }
        | AppError::BeanNotUpdated { .. }
        | AppError::UploadNotFound => StatusCode::NOT_FOUND,
        AppError::UploadInProgress => StatusCode::CONFLICT,
        AppError::Database { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        AppError::Storage(_) | AppError::Reader(_) => StatusCode::BAD_GATEWAY,
    }
}

fn client_message(err: &AppError) -> String {
    match err {
        AppError::NameRequired => {
            "A name is required. Send the coffee name as a non-empty string.".to_owned()
        }
        AppError::UploadIdRequired => {
            "An upload id is required. Use the id returned when the upload URL was created."
                .to_owned()
        }
        AppError::ImageMissing => {
            "The image is not in S3 yet. PUT the photo to the upload URL, then use the same id."
                .to_owned()
        }
        AppError::UnsupportedImageType => {
            "The image type must be jpeg, png, webp, or heic.".to_owned()
        }
        AppError::BeanNotFound { id } => {
            format!("No coffee with id {id} was found for this user.")
        }
        AppError::BeanNotUpdated { id } => format!(
            "No coffee with id {id} was found for this user, so it was not updated."
        ),
        AppError::UploadNotFound => {
            "No unused upload ticket exists for this id. It may already have been claimed, or it belongs to another user."
                .to_owned()
        }
        AppError::UploadInProgress => "This upload is already being processed.".to_owned(),
        AppError::Database { action } => format!("The database failed while {action}."),
        AppError::Storage(fault) => storage_message(fault),
        AppError::Reader(fault) => {
            format!("Could not read the coffee bag. {}", reader_reason(fault))
        }
    }
}

fn storage_message(fault: &StorageFault) -> String {
    match fault {
        StorageFault::Unreachable { bucket } => format!(
            "Could not access S3 bucket {bucket}. Check the credentials, region, and that this key can list the bucket."
        ),
        StorageFault::UploadUrl => "S3 failed while creating the upload URL.".to_owned(),
        StorageFault::ReadUrl => "S3 failed while creating a read URL for the image.".to_owned(),
        StorageFault::Head => {
            "S3 failed while checking whether the uploaded image exists.".to_owned()
        }
        StorageFault::DeleteObject => "S3 failed while deleting the image.".to_owned(),
        StorageFault::BadTtl => {
            "Could not prepare the S3 signature because the link lifetime is invalid.".to_owned()
        }
    }
}

fn reader_reason(fault: &ReaderFault) -> &'static str {
    match fault {
        ReaderFault::RequestRejected => "OpenRouter did not accept the request.",
        ReaderFault::BodyUnreadable => "OpenRouter's response body could not be read.",
        ReaderFault::Upstream => "OpenRouter returned an error instead of a coffee description.",
        ReaderFault::NoMessage => "OpenRouter's response had no message text.",
        ReaderFault::Empty => "OpenRouter returned an empty description.",
        ReaderFault::NoJson => "The model reply did not contain a JSON object.",
        ReaderFault::InvalidJson => "The model reply was not valid JSON.",
        ReaderFault::NotACoffee => "The model reply was not a coffee description.",
        ReaderFault::NameMissing => "The model reply did not include a coffee name.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_and_sentences_live_on_the_handler_seam() {
        let name = AppError::NameRequired;
        assert_eq!(status(&name), StatusCode::BAD_REQUEST);
        assert_eq!(
            client_message(&name),
            "A name is required. Send the coffee name as a non-empty string."
        );

        let missing = AppError::UploadNotFound;
        assert_eq!(status(&missing), StatusCode::NOT_FOUND);

        let busy = AppError::UploadInProgress;
        assert_eq!(status(&busy), StatusCode::CONFLICT);
        assert_eq!(
            client_message(&busy),
            "This upload is already being processed."
        );

        let database = AppError::Database {
            action: "listing coffees",
        };
        assert_eq!(status(&database), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            client_message(&database),
            "The database failed while listing coffees."
        );

        let reader = AppError::Reader(ReaderFault::NameMissing);
        assert_eq!(status(&reader), StatusCode::BAD_GATEWAY);
        assert_eq!(
            client_message(&reader),
            "Could not read the coffee bag. The model reply did not include a coffee name."
        );

        let storage = AppError::Storage(StorageFault::ReadUrl);
        assert_eq!(status(&storage), StatusCode::BAD_GATEWAY);
        assert_eq!(
            client_message(&storage),
            "S3 failed while creating a read URL for the image."
        );
    }
}
