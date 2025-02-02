// Implementation to convert reqwest::Response into ApiResponse
use axum::{
    extract::State,
    Json,
};

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{os_tooling::OsProcessGroup};
use crate::cache::{get_cached_data,Cache};


pub async fn get_processes(
    State(storage): State<Arc<Mutex<Cache<String, Vec<OsProcessGroup>>>>>,
) -> Json<Vec<Vec<OsProcessGroup>>> {
    let  cache = storage.lock().await;
    let data = get_cached_data(&*cache);
    Json(data)
}