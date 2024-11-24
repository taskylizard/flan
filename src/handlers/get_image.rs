use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
};
use bytes::Bytes;
use fred::{
    error::RedisError,
    prelude::{KeysInterface, RedisPool},
    types::{Expiration, SetOptions},
};
use image::{ImageFormat, ImageOutputFormat};
use s3::Bucket;
use serde::Deserialize;
use std::io::Cursor;
use tracing::{debug, error};

#[derive(Debug, Deserialize)]
pub struct ImageParams {
    #[serde(default)]
    width: Option<u32>,
    #[serde(default)]
    height: Option<u32>,
    #[serde(default)]
    quality: Option<u8>,
    #[serde(default)]
    format: Option<String>,
}

#[derive(Debug)]
pub enum GetImageError {
    NotFound,
    DatabaseError(String),
    CompressionError(String),
    CacheError(String),
}

impl From<GetImageError> for StatusCode {
    fn from(error: GetImageError) -> StatusCode {
        match error {
            GetImageError::NotFound => StatusCode::NOT_FOUND,
            GetImageError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GetImageError::CompressionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GetImageError::CacheError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for GetImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetImageError::NotFound => write!(f, "Image not found"),
            GetImageError::DatabaseError(err) => write!(f, "Database error: {}", err),
            GetImageError::CompressionError(err) => write!(f, "Compression error: {}", err),
            GetImageError::CacheError(err) => write!(f, "Cache error: {}", err),
        }
    }
}

fn generate_cache_key(file_id: &str, params: &ImageParams) -> String {
    format!(
        "img:{}:w{:?}:h{:?}:q{:?}:f{:?}",
        file_id, params.width, params.height, params.quality, params.format
    )
}

async fn get_from_cache(pool: &RedisPool, cache_key: &str) -> Result<Option<Bytes>, RedisError> {
    debug!("Attempting to get image from cache with key: {}", cache_key);
    let cached = pool.get::<Option<Bytes>, _>(cache_key).await?;
    Ok(cached.map(Bytes::from))
}

async fn set_in_cache(
    pool: &RedisPool,
    cache_key: &str,
    data: &[u8],
    ttl_secs: i64,
) -> Result<(), RedisError> {
    debug!("Caching image with key: {}", cache_key);
    pool.set(
        cache_key,
        data,
        Some(Expiration::EX(ttl_secs)),
        Some(SetOptions::NX),
        false,
    )
    .await
}

async fn find_image_with_extension(
    bucket: &Bucket,
    file_id: &str,
) -> Result<(String, Bucket), GetImageError> {
    debug!("Searching for image with file_id: {}", file_id);

    // List all objects in the bucket with the file_id prefix
    let objects = bucket.list(file_id.to_string(), None).await.map_err(|e| {
        error!("Failed to list objects: {}", e);
        GetImageError::DatabaseError(e.to_string())
    })?;

    debug!("Found {} objects with prefix {}", objects.len(), file_id);

    // Find the first object that starts with our file_id
    let object = objects
        .first()
        .ok_or_else(|| {
            error!("No objects found with prefix {}", file_id);
            GetImageError::NotFound
        })?
        .contents
        .first()
        .ok_or_else(|| {
            error!("Empty contents for prefix {}", file_id);
            GetImageError::NotFound
        })?;

    debug!("Found object: {}", object.key);

    Ok((object.key.clone(), bucket.clone()))
}

fn process_image(data: &[u8], params: &ImageParams) -> Result<(Vec<u8>, String), GetImageError> {
    let img = image::load_from_memory(data)
        .map_err(|e| GetImageError::CompressionError(format!("Failed to load image: {}", e)))?;

    let processed = if params.width.is_some() || params.height.is_some() {
        let width = params.width.unwrap_or(img.width());
        let height = params.height.unwrap_or(img.height());
        img.resize(width, height, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };

    // Determine output format
    let format = match params.format.as_deref() {
        Some("jpeg") | Some("jpg") => (
            ImageOutputFormat::Jpeg(params.quality.unwrap_or(80)),
            "image/jpeg",
        ),
        Some("png") => (ImageOutputFormat::Png, "image/png"),
        Some("webp") => (ImageOutputFormat::WebP, "image/webp"),
        _ => {
            // Default to original format or JPEG
            match image::guess_format(data).unwrap_or(ImageFormat::Jpeg) {
                ImageFormat::Jpeg => (
                    ImageOutputFormat::Jpeg(params.quality.unwrap_or(80)),
                    "image/jpeg",
                ),
                ImageFormat::Png => (ImageOutputFormat::Png, "image/png"),
                ImageFormat::WebP => (ImageOutputFormat::WebP, "image/webp"),
                _ => (
                    ImageOutputFormat::Jpeg(params.quality.unwrap_or(80)),
                    "image/jpeg",
                ),
            }
        }
    };

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    processed
        .write_to(&mut cursor, format.0)
        .map_err(|e| GetImageError::CompressionError(format!("Failed to encode image: {}", e)))?;

    Ok((buffer, format.1.to_string()))
}

pub async fn get_image_handler(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Query(params): Query<ImageParams>,
) -> Result<(HeaderMap, Bytes), StatusCode> {
    debug!("Getting image with file_id: {}", file_id);
    // Generate cache key based on file_id and processing parameters
    let cache_key = generate_cache_key(&file_id, &params);

    // Try to get from cache first
    if let Ok(Some(cached_data)) = get_from_cache(&state.redis, &cache_key).await {
        debug!("Cache hit for key: {}", cache_key);
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_str("image/jpeg").unwrap(),
        );
        headers.insert(
            axum::http::header::CONTENT_LENGTH,
            HeaderValue::from_str(&cached_data.len().to_string()).unwrap(),
        );
        return Ok((headers, cached_data));
    }

    // If not in cache, get from storage
    let (object_name, bucket) = find_image_with_extension(&state.bucket, &file_id)
        .await
        .map_err(StatusCode::from)?;

    let object = bucket.get_object(&object_name).await.map_err(|e| {
        error!("Failed to get object: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let data = object.bytes();

    // Process image if any parameters are specified
    let (processed_data, content_type) = if params.width.is_some()
        || params.height.is_some()
        || params.quality.is_some()
        || params.format.is_some()
    {
        process_image(data, &params).map_err(|e| {
            error!("Image processing error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        // If no processing needed, determine content type from original
        let content_type = match object_name.split('.').last().unwrap_or("") {
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "svg" => "image/svg+xml",
            _ => "application/octet-stream",
        };
        (data.to_vec(), content_type.to_string())
    };

    // Cache the processed result
    if let Err(e) = set_in_cache(&state.redis, &cache_key, &processed_data, 3600).await {
        error!("Failed to cache image: {}", e);
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_str(&content_type).map_err(|e| {
            error!("Invalid content-type header value: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );
    headers.insert(
        axum::http::header::CONTENT_LENGTH,
        HeaderValue::from_str(&processed_data.len().to_string()).map_err(|e| {
            error!("Failed to create content-length header: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );

    Ok((headers, processed_data.into()))
}
