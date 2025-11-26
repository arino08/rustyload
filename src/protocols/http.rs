//! HTTP protocol implementation for load testing

use crate::protocols::{calculate_stats, LoadTestStats, RequestResult};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Method};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

/// Supported HTTP methods for load testing
#[derive(Debug, Clone, Default, PartialEq)]
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

/// HTTP-specific configuration
#[derive(Debug, Clone)]
pub struct HttpConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl HttpConfig {
    pub fn new(url: String) -> Self {
        Self {
            url,
            method: HttpMethod::GET,
            headers: HashMap::new(),
            body: None,
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
}

/// Fire a single HTTP request and return the result
pub async fn fire_single_request(client: &Client, config: &HttpConfig) -> RequestResult {
    let start = Instant::now();

    // Build the request
    let mut request_builder = client.request(config.method.to_reqwest_method(), &config.url);

    // Add custom headers
    for (key, value) in &config.headers {
        request_builder = request_builder.header(key, value);
    }

    // Add body if present
    if let Some(body_content) = &config.body {
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

/// Run an HTTP load test with the given configuration
pub async fn run_load_test(
    http_config: &HttpConfig,
    num_requests: u64,
    concurrency: u64,
    timeout_secs: u64,
) -> Result<LoadTestStats> {
    let client = Client::builder()
        .user_agent("rustyload/0.2")
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .context("Failed to build HTTP client")?;

    let client = Arc::new(client);
    let semaphore = Arc::new(Semaphore::new(concurrency as usize));
    let config = Arc::new(http_config.clone());

    // Create progress bar
    let pb = ProgressBar::new(num_requests);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );
    pb.set_message("Sending HTTP requests...");

    let overall_start = Instant::now();

    // Spawn all tasks
    let mut handles = Vec::with_capacity(num_requests as usize);

    for _ in 0..num_requests {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);
        let config = Arc::clone(&config);
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = fire_single_request(&client, &config).await;
            pb.inc(1);
            result
        });

        handles.push(handle);
    }

    // Collect results
    let mut results = Vec::with_capacity(num_requests as usize);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_from_str() {
        assert_eq!(HttpMethod::from_str("GET").unwrap(), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str("get").unwrap(), HttpMethod::GET);
        assert_eq!(HttpMethod::from_str("POST").unwrap(), HttpMethod::POST);
        assert_eq!(HttpMethod::from_str("PUT").unwrap(), HttpMethod::PUT);
        assert_eq!(HttpMethod::from_str("DELETE").unwrap(), HttpMethod::DELETE);
        assert_eq!(HttpMethod::from_str("PATCH").unwrap(), HttpMethod::PATCH);
        assert_eq!(HttpMethod::from_str("HEAD").unwrap(), HttpMethod::HEAD);
        assert!(HttpMethod::from_str("INVALID").is_err());
    }

    #[test]
    fn test_http_config_builder() {
        let config = HttpConfig::new("https://example.com".to_string())
            .with_method(HttpMethod::POST)
            .with_body(Some("test body".to_string()));

        assert_eq!(config.url, "https://example.com");
        assert_eq!(config.method, HttpMethod::POST);
        assert_eq!(config.body, Some("test body".to_string()));
    }

    #[test]
    fn test_http_config_with_headers() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let config = HttpConfig::new("https://api.example.com".to_string()).with_headers(headers);

        assert!(config.headers.contains_key("Authorization"));
    }
}
