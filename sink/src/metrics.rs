use axum::http::{header, StatusCode};
use axum::response::Response;
use lazy_static::lazy_static;
use prometheus::{register_gauge, register_histogram, Encoder, Gauge, Histogram, TextEncoder};
use std::time::SystemTime;

lazy_static! {
    pub static ref PROCESS_START_TIME: SystemTime = SystemTime::now();
    pub static ref PROCESS_UPTIME_SECONDS: Gauge = register_gauge!(
        "process_uptime_seconds",
        "Time since the sink started in seconds"
    )
    .expect("Failed to create process_uptime_seconds gauge");
    pub static ref HEAD_BLOCK_NUMBER: Gauge =
        register_gauge!("head_block_number", "Last processed block number")
            .expect("Failed to create head_block_number gauge");
    pub static ref HEAD_BLOCK_TIMESTAMP: Gauge =
        register_gauge!("head_block_timestamp", "Head block timestamp")
            .expect("Failed to create head_block_timestamp gauge");
    pub static ref HEAD_BLOCK_TIME_DRIFT: Gauge =
        register_gauge!("head_block_time_drift", "Head block time drift in seconds")
            .expect("Failed to create head_block_time_drift gauge");
    pub static ref BLOCK_PROCESSING_TIME: Histogram = register_histogram!(
        "block_processing_duration_seconds",
        "Time spent processing each block"
    )
    .expect("Failed to create block_processing_duration_seconds histogram");
}

pub async fn metrics_handler() -> Response<String> {
    if let Ok(duration) = PROCESS_START_TIME.elapsed() {
        PROCESS_UPTIME_SECONDS.set(duration.as_secs_f64());
    }

    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, encoder.format_type())
        .body(String::from_utf8(buffer).unwrap())
        .expect("Failed to create metrics response")
}
