use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderValue, Method},
    routing::get,
    Json, Router,
};
use clap::Parser;
use odin::{
    configuration::get_configuration,
    memory::{get_cached_data, Cache},
    monitor::SystemMonitor,
    os_tooling::OsProcessGroup,
    telemetry::{get_subscriber, init_subscriber},
    web::app::start_server,
};

use odin::web::routes::api::get_processes;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
#[tokio::main]
async fn main() {
    let blob_storage: Cache<String, Vec<OsProcessGroup>> = Cache::new(60);
    let storage = Arc::new(Mutex::new(blob_storage));
    start_server(storage).await
}
