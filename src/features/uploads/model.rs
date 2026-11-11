use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct UploadTicket {
    pub(crate) id: String,
    pub(crate) upload_url: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ClaimedUpload {
    pub(crate) key: String,
}
