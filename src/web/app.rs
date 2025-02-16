use std::sync::Arc;

use axum::{
    http::{HeaderValue, Method},
    routing::get,
    Router,
};

use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::{cache::Cache, monitor::SystemMonitor, os_tooling::process::OsProcessGroup};

use super::routes::api::get_processes;



pub async fn start_server(storage: SystemMonitor) {
    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET])
        .allow_headers(Any);
    let app = Router::new()
        .route("/processes", get(get_processes))
        .layer(cors)
        .with_state(storage);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
