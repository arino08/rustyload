#![allow(clippy::upper_case_acronyms)]

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Method};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

/// Supported HTTP methods for load testing
#[derive(Debug, Clone, Default)]
pub enum HttpMethod {
    #[default]
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
}

impl HttpMethod {
    /// Parse a string into an HttpMethod
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            "PUT" => Ok(HttpMethod::PUT),
            "DELETE" => Ok(HttpMethod::DELETE),
            "PATCH" => Ok(HttpMethod::PATCH),
            "HEAD" => Ok(HttpMethod::HEAD),
            _ => Err(format!("Unsupported HTTP method: {}", s)),
        }
    }

    /// Convert to reqwest::Method
    fn to_reqwest_method(&self) -> Method {
        match self {
            HttpMethod::GET => Method::GET,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::DELETE => Method::DELETE,
            HttpMethod::PATCH => Method::PATCH,
            HttpMethod::HEAD => Method::HEAD,
        }
    }
}

/// Configuration for the load test
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub num_requests: u64,
    pub concurrency: u64,
    pub timeout_secs: u64,
}

impl LoadTestConfig {
    pub fn new(url: String, num_requests: u64, concurrency: u64) -> Self {
        Self {
            url,
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
            num_requests,
            concurrency,
            timeout_secs: 30,
        }
    }

    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.method = method;
        self
    }

    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_body(mut self, body: Option<String>) -> Self {
        self.body = body;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestResult {
    pub duration: u128,
    pub status: u16,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct LoadTestStats {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_duration: u128,
    pub min_latency: u128,
    pub max_latency: u128,
    pub avg_latency: f64,
    pub p50: u128,
    pub p95: u128,
    pub p99: u128,
    pub requests_per_second: f64,
}

pub async fn fire_single_request(
    client: &Client,
    url: &str,
    method: &HttpMethod,
    headers: &HashMap<String, String>,
    body: &Option<String>,
) -> RequestResult {
    let start = Instant::now();

    // Build the request
    let mut request_builder = client.request(method.to_reqwest_method(), url);

    // Add custom headers
    for (key, value) in headers {
        request_builder = request_builder.header(key, value);
    }

    // Add body if present
    if let Some(body_content) = body {
        request_builder = request_builder.body(body_content.clone());
    }

    // Send the request
    match request_builder.send().await {
        Ok(response) => {
            let duration = start.elapsed().as_millis();
            let status = response.status().as_u16();
            let success = response.status().is_success();

            RequestResult {
                duration,
                status,
                success,
                error: None,
            }
        }
        Err(e) => {
            let duration = start.elapsed().as_millis();
            RequestResult {
                duration,
                status: 0,
                success: false,
                error: Some(e.to_string()),
            }
        }
    }
}

pub async fn run_load_test(config: LoadTestConfig) -> Result<LoadTestStats> {
    let client = Client::builder()
        .user_agent("rustyload/0.1")
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()
        .context("Failed to build HTTP client")?;

    let client = Arc::new(client);
    let semaphore = Arc::new(Semaphore::new(config.concurrency as usize));
    let method = Arc::new(config.method);
    let headers = Arc::new(config.headers);
    let body = Arc::new(config.body);

    // Create progress bar
    let pb = ProgressBar::new(config.num_requests);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );
    pb.set_message("Sending requests...");

    let overall_start = Instant::now();

    // Spawn all tasks
    let mut handles = Vec::with_capacity(config.num_requests as usize);

    for _ in 0..config.num_requests {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);
        let url = config.url.clone();
        let method = Arc::clone(&method);
        let headers = Arc::clone(&headers);
        let body = Arc::clone(&body);
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = fire_single_request(&client, &url, &method, &headers, &body).await;
            pb.inc(1);
            result
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::with_capacity(config.num_requests as usize);
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    let total_duration = overall_start.elapsed().as_millis();

    pb.finish_with_message("Complete!");

    // Calculate statistics
    let stats = calculate_stats(&results, total_duration);

    Ok(stats)
}

fn calculate_stats(results: &[RequestResult], total_duration: u128) -> LoadTestStats {
    let total_requests = results.len() as u64;
    let successful_requests = results.iter().filter(|r| r.success).count() as u64;
    let failed_requests = total_requests - successful_requests;

    // Get latencies from successful requests for percentile calculation
    let mut latencies: Vec<u128> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.duration)
        .collect();

    // Sort for percentile calculation
    latencies.sort_unstable();

    let (min_latency, max_latency, avg_latency, p50, p95, p99) = if latencies.is_empty() {
        (0, 0, 0.0, 0, 0, 0)
    } else {
        let min = *latencies.first().unwrap();
        let max = *latencies.last().unwrap();
        let sum: u128 = latencies.iter().sum();
        let avg = sum as f64 / latencies.len() as f64;

        let p50 = percentile(&latencies, 50.0);
        let p95 = percentile(&latencies, 95.0);
        let p99 = percentile(&latencies, 99.0);

        (min, max, avg, p50, p95, p99)
    };

    let requests_per_second = if total_duration > 0 {
        (total_requests as f64 / total_duration as f64) * 1000.0
    } else {
        0.0
    };

    LoadTestStats {
        total_requests,
        successful_requests,
        failed_requests,
        total_duration,
        min_latency,
        max_latency,
        avg_latency,
        p50,
        p95,
        p99,
        requests_per_second,
    }
}

fn percentile(sorted_data: &[u128], pct: f64) -> u128 {
    if sorted_data.is_empty() {
        return 0;
    }

    let len = sorted_data.len();
    let rank = (pct / 100.0) * (len - 1) as f64;
    let lower = rank.floor() as usize;
    let upper = rank.ceil() as usize;

    if lower == upper || upper >= len {
        sorted_data[lower.min(len - 1)]
    } else {
        let weight = rank - lower as f64;
        let lower_val = sorted_data[lower] as f64;
        let upper_val = sorted_data[upper] as f64;
        (lower_val + weight * (upper_val - lower_val)) as u128
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== HttpMethod Tests ====================

    #[test]
    fn test_http_method_from_str() {
        assert!(matches!(HttpMethod::from_str("GET"), Ok(HttpMethod::GET)));
        assert!(matches!(HttpMethod::from_str("get"), Ok(HttpMethod::GET)));
        assert!(matches!(HttpMethod::from_str("POST"), Ok(HttpMethod::POST)));
        assert!(matches!(HttpMethod::from_str("post"), Ok(HttpMethod::POST)));
        assert!(matches!(HttpMethod::from_str("PUT"), Ok(HttpMethod::PUT)));
        assert!(matches!(
            HttpMethod::from_str("DELETE"),
            Ok(HttpMethod::DELETE)
        ));
        assert!(matches!(
            HttpMethod::from_str("PATCH"),
            Ok(HttpMethod::PATCH)
        ));
        assert!(matches!(HttpMethod::from_str("HEAD"), Ok(HttpMethod::HEAD)));
    }

    #[test]
    fn test_http_method_invalid() {
        assert!(HttpMethod::from_str("INVALID").is_err());
        assert!(HttpMethod::from_str("").is_err());
        assert!(HttpMethod::from_str("CONNECT").is_err());
    }

    // ==================== LoadTestConfig Tests ====================

    #[test]
    fn test_load_test_config_new() {
        let config = LoadTestConfig::new("http://example.com".to_string(), 100, 10);
        assert_eq!(config.url, "http://example.com");
        assert_eq!(config.num_requests, 100);
        assert_eq!(config.concurrency, 10);
        assert!(matches!(config.method, HttpMethod::GET));
        assert!(config.headers.is_empty());
        assert!(config.body.is_none());
    }

    #[test]
    fn test_load_test_config_builder() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let config = LoadTestConfig::new("http://example.com".to_string(), 100, 10)
            .with_method(HttpMethod::POST)
            .with_headers(headers)
            .with_body(Some("{\"key\": \"value\"}".to_string()))
            .with_timeout(60);

        assert!(matches!(config.method, HttpMethod::POST));
        assert_eq!(config.headers.len(), 1);
        assert!(config.body.is_some());
        assert_eq!(config.timeout_secs, 60);
    }

    // ==================== Percentile Tests ====================

    #[test]
    fn test_percentile_empty_data() {
        let data: Vec<u128> = vec![];
        assert_eq!(percentile(&data, 50.0), 0);
        assert_eq!(percentile(&data, 95.0), 0);
        assert_eq!(percentile(&data, 99.0), 0);
    }

    #[test]
    fn test_percentile_single_element() {
        let data = vec![100];
        assert_eq!(percentile(&data, 0.0), 100);
        assert_eq!(percentile(&data, 50.0), 100);
        assert_eq!(percentile(&data, 100.0), 100);
    }

    #[test]
    fn test_percentile_two_elements() {
        let data = vec![100, 200];
        assert_eq!(percentile(&data, 0.0), 100);
        assert_eq!(percentile(&data, 50.0), 150); // interpolated
        assert_eq!(percentile(&data, 100.0), 200);
    }

    #[test]
    fn test_percentile_sorted_data() {
        // 10 elements: indices 0-9
        let data: Vec<u128> = (1..=10).map(|x| x * 10).collect(); // [10, 20, 30, ..., 100]

        // p50 should be around the middle (between 50 and 60)
        let p50 = percentile(&data, 50.0);
        assert!((50..=60).contains(&p50), "p50 was {}", p50);

        // p90 should be near the end
        let p90 = percentile(&data, 90.0);
        assert!((80..=100).contains(&p90), "p90 was {}", p90);
    }

    #[test]
    fn test_percentile_100_elements() {
        // Create 100 elements: [1, 2, 3, ..., 100]
        let data: Vec<u128> = (1..=100).collect();

        // p50 should be ~50
        let p50 = percentile(&data, 50.0);
        assert!((49..=51).contains(&p50), "p50 was {}", p50);

        // p95 should be ~95
        let p95 = percentile(&data, 95.0);
        assert!((94..=96).contains(&p95), "p95 was {}", p95);

        // p99 should be ~99
        let p99 = percentile(&data, 99.0);
        assert!((98..=100).contains(&p99), "p99 was {}", p99);
    }

    // ==================== Calculate Stats Tests ====================

    #[test]
    fn test_calculate_stats_empty_results() {
        let results: Vec<RequestResult> = vec![];
        let stats = calculate_stats(&results, 1000);

        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.min_latency, 0);
        assert_eq!(stats.max_latency, 0);
        assert_eq!(stats.avg_latency, 0.0);
    }

    #[test]
    fn test_calculate_stats_all_successful() {
        let results = vec![
            RequestResult {
                duration: 100,
                status: 200,
                success: true,
                error: None,
            },
            RequestResult {
                duration: 200,
                status: 200,
                success: true,
                error: None,
            },
            RequestResult {
                duration: 300,
                status: 200,
                success: true,
                error: None,
            },
        ];

        let stats = calculate_stats(&results, 1000);

        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 3);
        assert_eq!(stats.failed_requests, 0);
        assert_eq!(stats.min_latency, 100);
        assert_eq!(stats.max_latency, 300);
        assert_eq!(stats.avg_latency, 200.0);
    }

    #[test]
    fn test_calculate_stats_with_failures() {
        let results = vec![
            RequestResult {
                duration: 100,
                status: 200,
                success: true,
                error: None,
            },
            RequestResult {
                duration: 50,
                status: 0,
                success: false,
                error: Some("timeout".to_string()),
            },
            RequestResult {
                duration: 200,
                status: 500,
                success: false,
                error: None,
            },
            RequestResult {
                duration: 150,
                status: 201,
                success: true,
                error: None,
            },
        ];

        let stats = calculate_stats(&results, 2000);

        assert_eq!(stats.total_requests, 4);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 2);
        // Only successful requests count for latency stats
        assert_eq!(stats.min_latency, 100);
        assert_eq!(stats.max_latency, 150);
        assert_eq!(stats.avg_latency, 125.0);
    }

    #[test]
    fn test_calculate_stats_all_failed() {
        let results = vec![
            RequestResult {
                duration: 100,
                status: 0,
                success: false,
                error: Some("error".to_string()),
            },
            RequestResult {
                duration: 200,
                status: 500,
                success: false,
                error: None,
            },
        ];

        let stats = calculate_stats(&results, 500);

        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 2);
        // No successful requests, so latency stats are 0
        assert_eq!(stats.min_latency, 0);
        assert_eq!(stats.max_latency, 0);
        assert_eq!(stats.avg_latency, 0.0);
    }

    #[test]
    fn test_calculate_stats_requests_per_second() {
        let results = vec![
            RequestResult {
                duration: 100,
                status: 200,
                success: true,
                error: None,
            },
            RequestResult {
                duration: 100,
                status: 200,
                success: true,
                error: None,
            },
            RequestResult {
                duration: 100,
                status: 200,
                success: true,
                error: None,
            },
            RequestResult {
                duration: 100,
                status: 200,
                success: true,
                error: None,
            },
        ];

        // 4 requests in 2000ms = 2 req/sec
        let stats = calculate_stats(&results, 2000);
        assert_eq!(stats.requests_per_second, 2.0);

        // 4 requests in 1000ms = 4 req/sec
        let stats = calculate_stats(&results, 1000);
        assert_eq!(stats.requests_per_second, 4.0);
    }

    #[test]
    fn test_calculate_stats_percentiles() {
        // Create 100 successful requests with durations 1-100
        let results: Vec<RequestResult> = (1..=100)
            .map(|i| RequestResult {
                duration: i as u128,
                status: 200,
                success: true,
                error: None,
            })
            .collect();

        let stats = calculate_stats(&results, 10000);

        // p50 should be around 50
        assert!((49..=51).contains(&stats.p50), "p50 was {}", stats.p50);

        // p95 should be around 95
        assert!((94..=96).contains(&stats.p95), "p95 was {}", stats.p95);

        // p99 should be around 99
        assert!((98..=100).contains(&stats.p99), "p99 was {}", stats.p99);
    }

    // ==================== RequestResult Tests ====================

    #[test]
    fn test_request_result_clone() {
        let result = RequestResult {
            duration: 100,
            status: 200,
            success: true,
            error: None,
        };

        let cloned = result.clone();
        assert_eq!(cloned.duration, 100);
        assert_eq!(cloned.status, 200);
        assert!(cloned.success);
        assert!(cloned.error.is_none());
    }

    #[test]
    fn test_request_result_with_error() {
        let result = RequestResult {
            duration: 50,
            status: 0,
            success: false,
            error: Some("Connection refused".to_string()),
        };

        assert!(!result.success);
        assert_eq!(result.error, Some("Connection refused".to_string()));
    }
}
