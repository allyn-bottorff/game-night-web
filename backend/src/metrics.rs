// metrics.rs
use prometheus::{
    register_counter_vec, register_histogram_vec, register_int_counter, 
    Counter, CounterVec, Encoder, HistogramVec, IntCounter, TextEncoder
};
use std::sync::Mutex;
use once_cell::sync::Lazy;

// Define metrics
static REQUEST_COUNTER: Lazy<IntCounter> = Lazy::new(|| {
    register_int_counter!(
        "game_night_requests_total", 
        "Total number of requests"
    ).unwrap()
});

static ENDPOINT_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "game_night_endpoint_requests_total", 
        "Total requests by endpoint and method", 
        &["endpoint", "method"]
    ).unwrap()
});

static RESPONSE_TIME: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "game_night_response_time_seconds", 
        "Response time in seconds by endpoint", 
        &["endpoint"]
    ).unwrap()
});

static ERROR_COUNTER: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "game_night_errors_total", 
        "Total errors by endpoint and status code", 
        &["endpoint", "status"]
    ).unwrap()
});

// Initialize metrics
pub fn init_metrics() {
    // Just access the Lazy statics to initialize them
    Lazy::force(&REQUEST_COUNTER);
    Lazy::force(&ENDPOINT_COUNTER);
    Lazy::force(&RESPONSE_TIME);
    Lazy::force(&ERROR_COUNTER);
}

// Record a request
pub fn record_request(endpoint: &str, method: &str) {
    REQUEST_COUNTER.inc();
    ENDPOINT_COUNTER.with_label_values(&[endpoint, method]).inc();
}

// Record response time
pub fn record_response_time(endpoint: &str, duration_secs: f64) {
    RESPONSE_TIME.with_label_values(&[endpoint]).observe(duration_secs);
}

// Record an error
pub fn record_error(endpoint: &str, status_code: u16) {
    ERROR_COUNTER
        .with_label_values(&[endpoint, &status_code.to_string()])
        .inc();
}

// Get metrics as a string
pub fn get_metrics() -> String {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}