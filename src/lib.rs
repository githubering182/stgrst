pub mod core;
pub mod routes;
pub mod services;

pub const BUCKET_CHUNK_SIZE: usize = 512 * 1024;
pub const SKIP_BUFFER_SIZE: usize = 10 * 1024usize.pow(2);
pub const MAX_STREAM_LENGTH: usize = 10;
pub const MONGO_DB_NAME: &str = "storage";
