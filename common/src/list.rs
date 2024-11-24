use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ImageInfo {
    pub file_id: String,

    pub url: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize)]
pub struct ListImagesResponse {
    pub images: Vec<ImageInfo>,
}
