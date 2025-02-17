
use odin::{
    configuration::get_configuration, monitor::SystemMonitor, web::app::start_server,
};

#[tokio::main]
async fn main() {
    let settings = get_configuration().expect("Failed to read configuration.");
    let monitor = SystemMonitor::new(settings.clone());
    start_server(monitor).await
}
