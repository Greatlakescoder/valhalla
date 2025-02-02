// Implementation to convert reqwest::Response into ApiResponse
use axum::{
    extract::State,
    http::{HeaderValue, Method},
    routing::get,
    Json, Router,
};

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{memory::{get_cached_data, Cache}, os_tooling::OsProcessGroup};


pub async fn get_processes(
    State(storage): State<Arc<Mutex<Cache<String, Vec<OsProcessGroup>>>>>,
) -> Json<Vec<Vec<OsProcessGroup>>> {
    let  cache = storage.lock().await;
    let data = get_cached_data(&*cache);
    Json(data)
}