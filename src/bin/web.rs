use std::sync::Arc;

use odin::{
    cache::Cache,
    os_tooling::process::OsProcessGroup,
    web::app::start_server,
};

use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    let blob_storage: Cache<String, Vec<OsProcessGroup>> = Cache::new(60);
    let storage = Arc::new(Mutex::new(blob_storage));
    start_server(storage).await
}
