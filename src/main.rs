use axum::{
    extract::State,
    http::{HeaderValue, Method},
    routing::get,
    Json, Router,
};
use clap::Parser;
use odin::{
    configuration::get_configuration,
    memory::Cache,
    monitor::SystemMonitor,
    os_tooling::OsProcessGroup,
    telemetry::{get_subscriber, init_subscriber},
    web::app::start_server,
};

use std::{error::Error, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to send the request to
    #[arg(
        short,
        long,
        default_value = "http://ai-ollama.tail8c6aba.ts.net:11434/api/generate"
    )]
    url: String,

    /// Question to ask the model
    #[arg(short, long, default_value = "What is the origin of the name wesley")]
    query: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let args = Args::parse();
    // In your main function

    let subscriber = get_subscriber("odin".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    tracing::info!("System Monitor Starting");
    let settings = get_configuration().expect("Failed to read configuration.");
    let blob_storage: Cache<String, Vec<OsProcessGroup>> = Cache::new(60);
    let storage = Arc::new(Mutex::new(blob_storage));

    // Spawn monitoring task that runs every 30 seconds
    let monitor_cache = storage.clone();
    tokio::spawn(async move {
        loop {
            tracing::info!("System Monitor running");
            let monitor = SystemMonitor::new(settings.clone(), monitor_cache.clone());
            if let Err(e) = monitor.run().await {
                tracing::error!("Monitor error: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    let web_cache = storage.clone();
    tokio::spawn(async move {
        loop {
            tracing::info!("Web Server running");
            start_server(web_cache.clone()).await;
        }
    });

    // Keep main process running until ctrl-c
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");

    Ok(())
}
