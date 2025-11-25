use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
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

pub async fn fire_single_request(client: &Client, url: &str) -> RequestResult {
    let start = Instant::now();

    match client.get(url).send().await {
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

pub async fn run_load_test(
    url: &str,
    num_requests: u64,
    concurrency: u64,
) -> Result<LoadTestStats> {
    let client = Client::builder()
        .user_agent("rustyload/0.1")
        .build()
        .context("Failed to build HTTP client")?;

    let client = Arc::new(client);
    let semaphore = Arc::new(Semaphore::new(concurrency as usize));

    // Create progress bar
    let pb = ProgressBar::new(num_requests);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▓▒░  ")
    );
    pb.set_message("Sending requests...");

    let overall_start = Instant::now();

    // Spawn all tasks
    let mut handles = Vec::with_capacity(num_requests as usize);

    for _ in 0..num_requests {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&semaphore);
        let url = url.to_string();
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = fire_single_request(&client, &url).await;
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
