use prometheus::{Encoder, TextEncoder};
use super::collector::METRICS_COLLECTOR;

pub fn register_metrics() {
    // Metrics are registered in the lazy_static initialization
    log::info!("Metrics collector initialized");
}

pub fn get_metrics_text() -> Result<String, prometheus::Error> {
    let encoder = TextEncoder::new();
    let metric_families = METRICS_COLLECTOR.registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer).unwrap())
}
