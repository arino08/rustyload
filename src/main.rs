mod client;

use anyhow::Result;
use clap::Parser;
use colored::*;

#[derive(Parser, Debug)]
#[command(author, version, about = "RustyLoad - A simple HTTP load testing tool", long_about = None)]
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
        "  ⚡ Simple HTTP Load Testing Tool ⚡".yellow().bold()
    );
    println!();
}

fn print_config(url: &str, requests: u64, concurrency: u64) {
    println!("{}", "┌─────────────────────────────────────────┐".dimmed());
    println!(
        "{} {:<39} {}",
        "│".dimmed(),
        "Configuration".white().bold(),
        "│".dimmed()
    );
    println!("{}", "├─────────────────────────────────────────┤".dimmed());
    println!(
        "{} {:<15} {:<23} {}",
        "│".dimmed(),
        "Target:".green(),
        truncate_url(url, 23),
        "│".dimmed()
    );
    println!(
        "{} {:<15} {:<23} {}",
        "│".dimmed(),
        "Requests:".green(),
        requests,
        "│".dimmed()
    );
    println!(
        "{} {:<15} {:<23} {}",
        "│".dimmed(),
        "Concurrency:".green(),
        concurrency,
        "│".dimmed()
    );
    println!("{}", "└─────────────────────────────────────────┘".dimmed());
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
    println!("{}", "┌─────────────────────────────────────────┐".dimmed());
    println!(
        "{} {:<39} {}",
        "│".dimmed(),
        "📊 Results".white().bold(),
        "│".dimmed()
    );
    println!("{}", "├─────────────────────────────────────────┤".dimmed());

    // Request summary
    println!(
        "{} {:<20} {:<18} {}",
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
        "{} {:<20} {:<18} {}",
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
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "Failed:".cyan(),
        failed_colored,
        "│".dimmed()
    );

    println!("{}", "├─────────────────────────────────────────┤".dimmed());
    println!(
        "{} {:<39} {}",
        "│".dimmed(),
        "⏱️  Latency (ms)".white().bold(),
        "│".dimmed()
    );
    println!("{}", "├─────────────────────────────────────────┤".dimmed());

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "Min:".cyan(),
        format!("{} ms", stats.min_latency),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "Max:".cyan(),
        format!("{} ms", stats.max_latency),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "Average:".cyan(),
        format!("{:.2} ms", stats.avg_latency),
        "│".dimmed()
    );

    println!("{}", "├─────────────────────────────────────────┤".dimmed());
    println!(
        "{} {:<39} {}",
        "│".dimmed(),
        "📈 Percentiles".white().bold(),
        "│".dimmed()
    );
    println!("{}", "├─────────────────────────────────────────┤".dimmed());

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "p50 (median):".magenta(),
        format!("{} ms", stats.p50).yellow(),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "p95:".magenta(),
        format!("{} ms", stats.p95).yellow(),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "p99:".magenta(),
        format!("{} ms", stats.p99).yellow(),
        "│".dimmed()
    );

    println!("{}", "├─────────────────────────────────────────┤".dimmed());
    println!(
        "{} {:<39} {}",
        "│".dimmed(),
        "🚀 Throughput".white().bold(),
        "│".dimmed()
    );
    println!("{}", "├─────────────────────────────────────────┤".dimmed());

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "Requests/sec:".green(),
        format!("{:.2}", stats.requests_per_second).green().bold(),
        "│".dimmed()
    );

    println!(
        "{} {:<20} {:<18} {}",
        "│".dimmed(),
        "Total time:".green(),
        format!("{} ms", stats.total_duration),
        "│".dimmed()
    );

    println!("{}", "└─────────────────────────────────────────┘".dimmed());
    println!();
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    print_banner();
    print_config(&args.url, args.requests, args.concurrency);

    println!("{}", "Starting load test...".yellow());
    println!();

    let stats = client::run_load_test(&args.url, args.requests, args.concurrency).await?;

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
