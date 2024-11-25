use crate::handlers::create_router;
use crate::layers::logger::LoggingMiddleware;
use crate::state::AppState;
use color_eyre::eyre;
use color_eyre::eyre::WrapErr;
use common::{config::AppConfig, Config};
use dotenvy::dotenv;
use eyre::{Report, Result};
use fred::{
    prelude::ClientLike,
    types::{Builder, ReconnectPolicy, RedisConfig},
};
use s3::{
    creds::Credentials,
    region::Region,
    {Bucket, BucketConfiguration},
};
use std::{process::ExitCode, sync::Arc, time::Duration};
use time::macros::format_description;
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer};
use tower_layer::layer_fn;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{
    fmt::{format::FmtSpan, layer, time::UtcTime},
    EnvFilter,
};

#[allow(warnings, unused)]
mod db;
mod handlers;
mod layers;
mod state;

#[tokio::main]
async fn main() -> Result<ExitCode, Report> {
    dotenv().ok();
    // Initialize tracing
    let fmt_layer = layer()
        .with_target(false)
        .with_timer(UtcTime::new(format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        )))
        .with_span_events(FmtSpan::FULL)
        .compact();

    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(
            "https://github.com/taskylizard/flan",
            "/issues/new"
        ))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .issue_filter(|kind| match kind {
            color_eyre::ErrorKind::NonRecoverable(_) => false,
            color_eyre::ErrorKind::Recoverable(_) => true,
        })
        .install()?;

    let config: AppConfig = Config::builder()
        .env()
        .file("config.toml")
        .load()
        .context("Failed to load config")?;

    let prisma = db::new_client_with_url(&config.database_url)
        .await
        .with_context(|| "Failed to connect to database")?;
    info!("Connected to database");

    let region = Region::Custom {
        region: "eu-central-1".to_owned(),
        endpoint: config.minio.endpoint,
    };
    let credentials = Credentials::new(
        Some(&config.minio.access_key),
        Some(&config.minio.secret_key),
        None,
        None,
        None,
    )?;

    let mut bucket = Bucket::new(
        &config.minio.bucket_name,
        region.clone(),
        credentials.clone(),
    )?
    .with_path_style();

    let bkt_exists = bucket.exists().await?;

    // Create bucket if it doesn't exist
    if !bkt_exists {
        bucket = Bucket::create_with_path_style(
            &config.minio.bucket_name,
            region,
            credentials,
            BucketConfiguration::default(),
        )
        .await
        .with_context(|| format!("Failed to create bucket {}", config.minio.bucket_name))?
        .bucket;
    }
    info!("Connected to Minio");

    let redis_config =
        RedisConfig::from_url(&config.redis.url).expect("Failed to create redis config from url");
    let redis_pool = Builder::from_config(redis_config)
        .with_connection_config(|config| {
            config.connection_timeout = Duration::from_secs(10);
        })
        // use exponential backoff, starting at 100 ms and doubling on each failed attempt up to 30 sec
        .set_policy(ReconnectPolicy::new_exponential(0, 100, 30_000, 2))
        .build_pool(config.redis.pool_size)
        .expect("Failed to create redis pool");

    redis_pool.init().await.expect("Failed to connect to redis");
    info!("Connected to Redis");

    let state = AppState {
        bucket,
        db: Arc::new(prisma),
        admin_key: config.admin_key,
        redis: redis_pool,
    };

    let app = create_router()
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(CompressionLayer::new())
        .layer(layer_fn(LoggingMiddleware))
        .with_state(state);

    // Run our server based on configuration values provided
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.address, config.port))
        .await
        .with_context(|| format!("Failed to bind to {}:{}", config.address, config.port))?;

    info!("🍃 Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .context("Failed to serve")?;

    Ok(ExitCode::SUCCESS)
}
