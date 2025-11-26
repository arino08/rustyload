//! Protocol abstraction module for supporting multiple load testing targets
//!
//! This module provides a common interface for different protocols (HTTP, FlashKV, etc.)

pub mod flashkv;
pub mod http;

use std::collections::HashMap;

/// Supported protocols for load testing
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Protocol {
    #[default]
    Http,
    FlashKV,
}

impl Protocol {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "http" | "https" => Ok(Protocol::Http),
            "flashkv" | "kv" | "tcp" => Ok(Protocol::FlashKV),
            _ => Err(format!("Unsupported protocol: {}", s)),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Protocol::Http => "HTTP/HTTPS",
            Protocol::FlashKV => "FlashKV (TCP)",
        }
    }
}

/// Common result structure for any protocol request
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RequestResult {
    /// Duration of the request in milliseconds
    pub duration: u128,
    /// Protocol-specific status code (HTTP status, or custom for TCP)
    pub status: u16,
    /// Whether the request was successful
    pub success: bool,
    /// Error message if the request failed
    pub error: Option<String>,
}

/// Statistics from a load test run
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

/// Unified configuration for load testing any protocol
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    /// The protocol to use
    pub protocol: Protocol,
    /// Number of requests to send
    pub num_requests: u64,
    /// Number of concurrent requests
    pub concurrency: u64,
    /// Timeout in seconds
    pub timeout_secs: u64,
    /// HTTP-specific configuration
    pub http_config: Option<http::HttpConfig>,
    /// FlashKV-specific configuration
    pub flashkv_config: Option<flashkv::FlashKVConfig>,
}

#[allow(dead_code)]
impl LoadTestConfig {
    pub fn new_http(url: String, num_requests: u64, concurrency: u64) -> Self {
        Self {
            protocol: Protocol::Http,
            num_requests,
            concurrency,
            timeout_secs: 30,
            http_config: Some(http::HttpConfig {
                url,
                method: http::HttpMethod::GET,
                headers: HashMap::new(),
                body: None,
            }),
            flashkv_config: None,
        }
    }

    pub fn new_flashkv(
        host: String,
        port: u16,
        commands: Vec<flashkv::FlashKVCommand>,
        num_requests: u64,
        concurrency: u64,
    ) -> Self {
        Self {
            protocol: Protocol::FlashKV,
            num_requests,
            concurrency,
            timeout_secs: 30,
            http_config: None,
            flashkv_config: Some(flashkv::FlashKVConfig {
                host,
                port,
                commands,
                use_random_keys: false,
                key_prefix: "key".to_string(),
                key_range: 1000,
            }),
        }
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Get display URL/address for the config
    pub fn display_target(&self) -> String {
        match self.protocol {
            Protocol::Http => self
                .http_config
                .as_ref()
                .map(|c| c.url.clone())
                .unwrap_or_else(|| "unknown".to_string()),
            Protocol::FlashKV => self
                .flashkv_config
                .as_ref()
                .map(|c| format!("{}:{}", c.host, c.port))
                .unwrap_or_else(|| "unknown".to_string()),
        }
    }
}

/// Calculate statistics from request results
pub fn calculate_stats(results: &[RequestResult], total_duration: u128) -> LoadTestStats {
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

    #[test]
    fn test_protocol_from_str() {
        assert_eq!(Protocol::from_str("http").unwrap(), Protocol::Http);
        assert_eq!(Protocol::from_str("HTTP").unwrap(), Protocol::Http);
        assert_eq!(Protocol::from_str("https").unwrap(), Protocol::Http);
        assert_eq!(Protocol::from_str("flashkv").unwrap(), Protocol::FlashKV);
        assert_eq!(Protocol::from_str("kv").unwrap(), Protocol::FlashKV);
        assert_eq!(Protocol::from_str("tcp").unwrap(), Protocol::FlashKV);
        assert!(Protocol::from_str("invalid").is_err());
    }

    #[test]
    fn test_calculate_stats_empty() {
        let results: Vec<RequestResult> = vec![];
        let stats = calculate_stats(&results, 1000);
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.successful_requests, 0);
        assert_eq!(stats.failed_requests, 0);
    }

    #[test]
    fn test_calculate_stats_with_results() {
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
                duration: 50,
                status: 0,
                success: false,
                error: Some("timeout".to_string()),
            },
        ];
        let stats = calculate_stats(&results, 1000);
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.successful_requests, 2);
        assert_eq!(stats.failed_requests, 1);
        assert_eq!(stats.min_latency, 100);
        assert_eq!(stats.max_latency, 200);
    }
}
