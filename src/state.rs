use crate::db::PrismaClient;
use fred::clients::RedisPool;
use s3::Bucket;
use std::sync::Arc;

pub type Database = Arc<PrismaClient>;

#[derive(Clone)]
pub struct AppState {
    pub bucket: Bucket,
    pub db: Database,
    pub redis: RedisPool,
    pub admin_key: String,
}
