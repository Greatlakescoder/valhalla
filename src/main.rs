use clap::Parser;
use metrics::counter;
use odin::{
    configuration::get_configuration,
    monitor::SystemMonitor,
    telemetry::{get_subscriber, init_subscriber},
};
use std::{error::Error, thread::sleep, time::Duration};

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

// Implementation to convert reqwest::Response into ApiResponse

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // let args = Args::parse();

    let subscriber = get_subscriber("odin".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    tracing::info!("System Monitor Starting");
    let settings = get_configuration().expect("Failed to read configuration.");
    
    // Spawn monitoring task that runs every 30 seconds
    tokio::spawn(async move {
        loop {
            tracing::info!("System Monitor running");
            let monitor = SystemMonitor::new(settings.clone());
            if let Err(e) = monitor.run().await {
                tracing::error!("Monitor error: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });

    // Keep main process running until ctrl-c
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down");
    
    Ok(())
}
