use crate::image::{transform, Dimension, Format, Height, Operation, Width};
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
use image::{guess_format, ImageFormat};
use s3::Bucket;
use serde::Deserialize;
use tracing::{debug, error};
use utoipa::IntoParams;

#[derive(Debug, Deserialize, IntoParams)]
pub struct GetImageParams {
    /// Width of the image
    width: Option<Width>,
    /// Height of the image
    height: Option<Height>,
    /// Quality of the image
    quality: Option<u8>,
    /// Format of the image
    format: Option<Format>,
}

#[derive(Debug)]
pub enum GetImageError {
    NotFound,
    DatabaseError(String),
    CompressionError(String),
    CacheError(String),
    ResizeEmptyDimension(String),
}

impl From<GetImageError> for StatusCode {
    fn from(error: GetImageError) -> StatusCode {
        match error {
            GetImageError::NotFound => StatusCode::NOT_FOUND,
            GetImageError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GetImageError::CompressionError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GetImageError::CacheError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            GetImageError::ResizeEmptyDimension(_) => StatusCode::INTERNAL_SERVER_ERROR,
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
            GetImageError::ResizeEmptyDimension(err) => {
                write!(f, "Resize empty dimension: {}", err)
            }
        }
    }
}

fn generate_cache_key(file_id: &str, params: &GetImageParams) -> String {
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
fn guess_content_type(image: &[u8]) -> Result<String, GetImageError> {
    let format = guess_format(image).map_err(|e| {
        GetImageError::CompressionError(format!("Failed to guess image format: {}", e))
    })?;

    // only supported types are https://www.iana.org/assignments/media-types/media-types.xhtml#image
    match format {
        ImageFormat::Png => Ok("image/png".into()),
        ImageFormat::Jpeg => Ok("image/jpeg".into()),
        ImageFormat::Gif => Ok("image/gif".into()),
        ImageFormat::WebP => Ok("image/webp".into()),
        ImageFormat::Tiff => Ok("image/tiff".into()),
        ImageFormat::Bmp => Ok("image/bmp".into()),
        ImageFormat::Ico => Ok("image/x-icon".into()), // https://stackoverflow.com/a/28300054
        ImageFormat::Avif => Ok("image/avif".into()),
        _ => Err(GetImageError::CompressionError(format!(
            "Unsupported image format: {:?}",
            format
        ))),
    }
}

fn get_operations(opts: &GetImageParams) -> Vec<Operation> {
    let mut operations = Vec::with_capacity(2);
    if let Some(f) = opts.format {
        operations.push(Operation::Convert(f));
    }
    if let Some(q) = opts.quality {
        operations.push(Operation::Quality(q));
    }
    match (opts.width, opts.height) {
        (None, None) => (),
        _ => operations.push(Operation::Resize(Dimension(opts.width, opts.height))),
    }
    operations
}

/// Get an image
#[utoipa::path(
    get,
    path = "/:file_id",
    tag = "Image",
    params(
      GetImageParams,
    ),
    description = "Get an image (optionally resized), returns the image headers and bytes",
    responses(
            (status = 200, description = "Image retrieved successfully", 
                headers(
                ("Content-Type" = String, description = "MIME type of the file"),
                ("Content-Length" = String, description = "Length of the file")
            ),
            ),
        (status = 400, description = "Invalid credentials"),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Image not found"),
        (status = 500, description = "Internal server error" )
    ),
    security((),
        ("access_key" = []),
        ("admin_key" = [])
    )
)]
pub async fn get_image_handler(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Query(params): Query<GetImageParams>,
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
        headers.insert(
            axum::http::header::CACHE_CONTROL,
            HeaderValue::from_str("max-age=31536000").unwrap(),
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

    // Prepare operations based on query parameters
    let operations: Vec<Operation> = get_operations(&params);

    // Transform image if any operations are specified
    let processed_data = if !operations.is_empty() {
        transform(data, &operations).map_err(|e| {
            error!("Image transformation error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    } else {
        data.to_vec()
    };

    // Determine content type
    let content_type = params
        .format
        .map(|f| f.content_type().to_string())
        .unwrap_or_else(|| guess_content_type(data).expect("Failed to guess content type"));

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
    headers.insert(
        axum::http::header::CACHE_CONTROL,
        HeaderValue::from_str("max-age=31536000").map_err(|e| {
            error!("Failed to create cache-control header: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?,
    );

    Ok((headers, processed_data.into()))
}
