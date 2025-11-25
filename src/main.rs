mod client;

use anyhow::Result;
use clap::Parser;
use colored::*;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about = "RustyLoad - A blazingly fast HTTP load testing tool", long_about = None)]
struct Args {
    /// Target URL to test
    #[clap(short, long)]
    url: String,

    /// Number of requests to send
    #[clap(short = 'n', long, default_value_t = 100)]
    requests: u64,

    /// Number of concurrent requests
    #[clap(short, long, default_value_t = 10)]
    concurrency: u64,

    /// HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD)
    #[clap(short, long, default_value = "GET")]
    method: String,

    /// Custom headers (can be used multiple times)
    /// Format: "Header-Name: Header-Value" or "Header-Name=Header-Value"
    #[clap(short = 'H', long = "header", value_name = "HEADER")]
    headers: Vec<String>,

    /// Request body (for POST, PUT, PATCH)
    #[clap(short, long)]
    body: Option<String>,

    /// Request timeout in seconds
    #[clap(short, long, default_value_t = 30)]
    timeout: u64,
}

fn print_banner() {
    println!();
    println!(
        "{}",
        r#"
  ____           _         _                    _
 |  _ \ _   _ __| |_ _   _| |    ___   __ _  __| |
 | |_) | | | / _` __| | | | |   / _ \ / _` |/ _` |
 |  _ <| |_| \__ \ |_| |_| | |__| (_) | (_| | (_| |
 |_| \_\\__,_|___/\__|\__, |_____\___/ \__,_|\__,_|
                      |___/
"#
        .cyan()
        .bold()
    );
    println!(
        "{}",
        "  ⚡ Blazingly Fast HTTP Load Testing Tool ⚡"
            .yellow()
            .bold()
    );
    println!();
}

fn print_config(config: &client::LoadTestConfig) {
    println!(
        "{}",
        "┌─────────────────────────────────────────────────┐".dimmed()
    );
    println!(
        "{} {:<47} {}",
        "│".dimmed(),
        "Configuration".white().bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );
    println!(
        "{} {:<15} {:<31} {}",
        "│".dimmed(),
        "Target:".green(),
        truncate_url(&config.url, 31),
        "│".dimmed()
    );
    println!(
        "{} {:<15} {:<31} {}",
        "│".dimmed(),
        "Method:".green(),
        format!("{:?}", config.method),
        "│".dimmed()
    );
    println!(
        "{} {:<15} {:<31} {}",
        "│".dimmed(),
        "Requests:".green(),
        config.num_requests,
        "│".dimmed()
    );
    println!(
        "{} {:<15} {:<31} {}",
        "│".dimmed(),
        "Concurrency:".green(),
        config.concurrency,
        "│".dimmed()
    );
    println!(
        "{} {:<15} {:<31} {}",
        "│".dimmed(),
        "Timeout:".green(),
        format!("{} seconds", config.timeout_secs),
        "│".dimmed()
    );

    if !config.headers.is_empty() {
        println!(
            "{}",
            "├─────────────────────────────────────────────────┤".dimmed()
        );
        println!(
            "{} {:<47} {}",
            "│".dimmed(),
            "Custom Headers".white().bold(),
            "│".dimmed()
        );
        for (key, value) in &config.headers {
            let header_str = format!("{}: {}", key, truncate_url(value, 20));
            println!(
                "{} {:<47} {}",
                "│".dimmed(),
                header_str.dimmed(),
                "│".dimmed()
            );
        }
    }

    if let Some(body) = &config.body {
        println!(
            "{}",
            "├─────────────────────────────────────────────────┤".dimmed()
        );
        println!(
            "{} {:<15} {:<31} {}",
            "│".dimmed(),
            "Body:".green(),
            truncate_url(body, 31),
            "│".dimmed()
        );
    }

    println!(
        "{}",
        "└─────────────────────────────────────────────────┘".dimmed()
    );
    println!();
}

fn truncate_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len - 3])
    }
}

fn print_results(stats: &client::LoadTestStats) {
    println!();
    println!(
        "{}",
        "┌─────────────────────────────────────────────────┐".dimmed()
    );
    println!(
        "{} {:<47} {}",
        "│".dimmed(),
        "📊 Results".white().bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );

    // Request summary
    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Total Requests:".cyan(),
        stats.total_requests,
        "│".dimmed()
    );

    let success_rate = if stats.total_requests > 0 {
        (stats.successful_requests as f64 / stats.total_requests as f64) * 100.0
    } else {
        0.0
    };

    let success_str = format!("{} ({:.1}%)", stats.successful_requests, success_rate);
    let success_colored = if success_rate >= 99.0 {
        success_str.green()
    } else if success_rate >= 95.0 {
        success_str.yellow()
    } else {
        success_str.red()
    };

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Successful:".cyan(),
        success_colored,
        "│".dimmed()
    );

    let failed_colored = if stats.failed_requests == 0 {
        stats.failed_requests.to_string().green()
    } else {
        stats.failed_requests.to_string().red()
    };

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Failed:".cyan(),
        failed_colored,
        "│".dimmed()
    );

    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );
    println!(
        "{} {:<47} {}",
        "│".dimmed(),
        "⏱️  Latency (ms)".white().bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Min:".cyan(),
        format!("{} ms", stats.min_latency),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Max:".cyan(),
        format!("{} ms", stats.max_latency),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Average:".cyan(),
        format!("{:.2} ms", stats.avg_latency),
        "│".dimmed()
    );

    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );
    println!(
        "{} {:<47} {}",
        "│".dimmed(),
        "📈 Percentiles".white().bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "p50 (median):".magenta(),
        format!("{} ms", stats.p50).yellow(),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "p95:".magenta(),
        format!("{} ms", stats.p95).yellow(),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "p99:".magenta(),
        format!("{} ms", stats.p99).yellow(),
        "│".dimmed()
    );

    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );
    println!(
        "{} {:<47} {}",
        "│".dimmed(),
        "🚀 Throughput".white().bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Requests/sec:".green(),
        format!("{:.2}", stats.requests_per_second).green().bold(),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<26} {}",
        "│".dimmed(),
        "Total time:".green(),
        format!("{} ms", stats.total_duration),
        "│".dimmed()
    );

    println!(
        "{}",
        "└─────────────────────────────────────────────────┘".dimmed()
    );
    println!();
}

fn parse_headers(header_strings: &[String]) -> Result<HashMap<String, String>> {
    let mut headers = HashMap::new();

    for header in header_strings {
        match client::parse_header(header) {
            Ok((key, value)) => {
                headers.insert(key, value);
            }
            Err(e) => {
                eprintln!("{}: {}", "Warning".yellow(), e);
            }
        }
    }

    Ok(headers)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse HTTP method
    let method = match client::HttpMethod::from_str(&args.method) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            std::process::exit(1);
        }
    };

    // Parse headers
    let headers = parse_headers(&args.headers)?;

    // Build configuration
    let config = client::LoadTestConfig::new(args.url, args.requests, args.concurrency)
        .with_method(method)
        .with_headers(headers)
        .with_body(args.body)
        .with_timeout(args.timeout);

    print_banner();
    print_config(&config);

    println!("{}", "Starting load test...".yellow());
    println!();

    let stats = client::run_load_test(config).await?;

    print_results(&stats);

    // Final summary line
    if stats.failed_requests == 0 {
        println!("{}", "✅ Load test completed successfully!".green().bold());
    } else {
        println!(
            "{}",
            format!(
                "⚠️  Load test completed with {} failed requests",
                stats.failed_requests
            )
            .yellow()
            .bold()
        );
    }
    println!();

    Ok(())
}
