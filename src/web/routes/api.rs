// Implementation to convert reqwest::Response into ApiResponse
use axum::{extract::State, Json};

use crate::{
    monitor::{MonitorOutput, SystemMonitor},
    ollama::{OllamaClient, ProcessScore},
};

pub async fn get_processes(State(monitor): State<SystemMonitor>) -> Json<MonitorOutput> {
    let snapshot = monitor.get_latest_snapshot().await;
    Json(snapshot)
}

pub async fn ollama_request(State(monitor): State<SystemMonitor>) -> Json<Vec<ProcessScore>> {
    let ollama_client = OllamaClient::new(monitor.clone().settings);
    let snapshot = monitor.get_latest_snapshot().await;
    let resp = ollama_client.analyze_system_monitor_output(&snapshot).await.unwrap();
    Json(resp)
}
