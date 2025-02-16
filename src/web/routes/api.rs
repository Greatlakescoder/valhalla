// Implementation to convert reqwest::Response into ApiResponse
use axum::{
    extract::State,
    Json,
};


use crate::monitor::{MonitorOutput, SystemMonitor};



pub async fn get_processes(
    State(monitor): State<SystemMonitor>,
) -> Json<MonitorOutput> {
    let snapshot = monitor.get_latest_snapshot().await;
    Json(snapshot)
}