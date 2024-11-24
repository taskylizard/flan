use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct UploadImageResponse {
    pub file_id: String,
    pub url: String,
}
