use std::fmt::Display;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StorageFault {
    Unreachable { bucket: String },
    UploadUrl,
    ReadUrl,
    Head,
    DeleteObject,
    BadTtl,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ReaderFault {
    RequestRejected,
    BodyUnreadable,
    Upstream,
    NoMessage,
    Empty,
    NoJson,
    InvalidJson,
    NotACoffee,
    NameMissing,
}

/// Outcomes from feature modules. The handler seam chooses the HTTP status and the client sentence.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AppError {
    NameRequired,
    UploadIdRequired,
    ImageMissing,
    UnsupportedImageType,
    BeanNotFound { id: i64 },
    BeanNotUpdated { id: i64 },
    UploadNotFound,
    UploadInProgress,
    Database { action: &'static str },
    Storage(StorageFault),
    Reader(ReaderFault),
}

pub(crate) fn database(action: &'static str, err: impl Display) -> AppError {
    tracing::error!(error = %err, "database failed while {action}");
    AppError::Database { action }
}
