use clap::Parser;
use odin::{
    cache::Cache, configuration::get_configuration, monitor::SystemMonitor, ollama::OllamaClient, os_tooling::process::OsProcessGroup, telemetry::{get_subscriber, init_subscriber}, web::app::start_server
};

use std::{error::Error, sync::Arc, time::Duration};
use tokio::sync::Mutex;

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

    let subscriber = get_subscriber("odin".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    tracing::info!("System Monitor Starting");
    let settings = get_configuration().expect("Failed to read configuration.");

    let monitor = SystemMonitor::new(settings.clone());
    tokio::spawn({
        let monitor = monitor.clone();
        async move {
            // So we spawn monitoring tasks which will spawn sub tasks for each monitor and then we await
            // which essentially is blocking so sub tasks can run forever until killed
            if let Err(e) = monitor.run().await {
                tracing::error!("Monitor error: {}", e);
            }
        }
    });

    let web_monitor = monitor.clone();
    tokio::spawn(async move {
        loop {
            tracing::info!("Web Server running");
            start_server(web_monitor.clone()).await;
        }
    });
    let ollama = OllamaClient::new(settings.clone());
    let monitor_clone = monitor.clone();
    tokio::spawn(async move {
        tracing::info!("Ollama analysis started");
        loop {
            match monitor_clone.run_analysis(ollama.clone()).await {
                Ok(_) => {
                    tracing::info!("Ollama analysis completed successfully");
                }
                Err(e) => {
                    tracing::error!("Ollama analysis failed: {}", e);
                    // Add a small delay before retrying to prevent spam
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });

    // Keep main process running until ctrl-c
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");

    Ok(())
}
