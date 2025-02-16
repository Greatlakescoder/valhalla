use std::sync::Arc;

use odin::{
    cache::Cache, configuration::get_configuration, monitor::SystemMonitor,
    os_tooling::process::OsProcessGroup, web::app::start_server,
};

use tokio::sync::Mutex;
#[tokio::main]
async fn main() {
    let settings = get_configuration().expect("Failed to read configuration.");
    let monitor = SystemMonitor::new(settings.clone());
    start_server(monitor).await
}
