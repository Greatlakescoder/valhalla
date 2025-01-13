use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_log::LogTracer;
use tracing_subscriber::fmt::{self, MakeWriter};
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use opentelemetry::sdk::trace::Config;

pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // Parse the env_filter or use the provided one
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

    // Setup the OpenTelemetry tracer
    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(name)
        .with_trace_config(Config::default())
        .install_simple()
        .expect("Failed to install OpenTelemetry tracer");

    // Create the logging layer using the provided sink
    let formatting_layer = fmt::layer()
        .with_writer(sink)
        .with_thread_ids(true)
        .with_target(true)
        .with_level(true);

    // Compose all layers
    Registry::default()
        .with(env_filter)
        .with(formatting_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // Redirect all log events to our subscriber
    LogTracer::init().expect("Failed to set logger");
    // Set the subscriber as the default
    set_global_default(subscriber).expect("Failed to set subscriber");
}
