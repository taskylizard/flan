use confique::Config;
use std::fmt::Debug;
use std::net::IpAddr;

#[derive(Debug, Config)]
pub struct AppConfig {
    /// Port to listen on.
    #[config(env = "PORT", default = 8080)]
    pub port: u16,

    /// Bind address.
    #[config(default = "127.0.0.1")]
    pub address: IpAddr,

    // Minio config, pointing to a local Minio instance.
    #[config(nested)]
    pub minio: MinioConfig,

    // Admin key for the admin to register accounts.
    #[config(default = "admin-key")]
    pub admin_key: String,

    /// Database URL.
    #[config(
        env = "DATABASE_URL",
        default = "postgresql://flan:supersecretpassword@localhost:6500/db?schema=public"
    )]
    pub database_url: String,

    #[config(nested)]
    pub redis: RedisConfig,
}

#[derive(Debug, Config)]
pub struct RedisConfig {
    #[config(env = "REDIS_URL", default = "redis://localhost:6379")]
    pub url: String,

    #[config(env = "REDIS_POOL_SIZE", default = 10)]
    pub pool_size: usize,
}

#[derive(Debug, Config)]
pub struct MinioConfig {
    /// Minio endpoint.
    #[config(env = "MINIO_ENDPOINT", default = "http://localhost:9000")]
    pub endpoint: String,

    /// Minio access key.
    #[config(env = "MINIO_ACCESS_KEY", default = "minioadmin")]
    pub access_key: String,

    /// Minio secret key.
    #[config(env = "MINIO_SECRET_KEY", default = "minioadmin")]
    pub secret_key: String,

    /// Minio bucket name.
    #[config(env = "MINIO_BUCKET_NAME", default = "images")]
    pub bucket_name: String,
}
