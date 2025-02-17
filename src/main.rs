use clap::Parser;
use odin::{
    cache::Cache,
    configuration::get_configuration,
    monitor::SystemMonitor,
    os_tooling::process::OsProcessGroup,
    telemetry::{get_subscriber, init_subscriber},
    web::app::start_server,
};

use std::{error::Error, sync::Arc};
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

    // Keep main process running until ctrl-c
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");

    Ok(())
}
