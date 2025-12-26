use crate::server::state::AppState;
use axum::extract::State;

pub async fn health_check() -> &'static str {
    "OK"
}

pub async fn metrics(State(state): State<AppState>) -> String {
    for collector in state.collectors.iter() {
        collector.collect().await.unwrap();
    }

    let metric_families = state.registry.gather();
    let encoder = prometheus::TextEncoder::new();

    encoder.encode_to_string(&metric_families).unwrap()
}
