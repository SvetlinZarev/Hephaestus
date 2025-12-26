use hephaestus::bootstrap::init_collectors;
use hephaestus::config::Configuration;
use hephaestus::logging::setup_logging;
use hephaestus::server::start_server;
use hephaestus::server::state::AppState;
use std::sync::Arc;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let configuration = Arc::new(Configuration::load()?);
    let _guard = setup_logging(&configuration.log)?;

    tracing::info!("Starting Hephaestus");
   
    let registry = prometheus::Registry::new();
    let collectors = Arc::new(init_collectors(&configuration.collectors, &registry)?);
    
    let state = AppState {
        configuration,
        registry,
        collectors,
    };

    start_server(state).await?;
    tracing::info!("Bye!");

    Ok(())
}
