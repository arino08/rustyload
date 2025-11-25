mod client;
mod interactive;

use anyhow::Result;
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};

#[derive(Parser, Debug)]
#[command(author, version, about = "RustyLoad - A blazingly fast HTTP load testing tool", long_about = None)]
struct Args {
    /// Target URL to test
    #[clap(short, long)]
    url: Option<String>,

    /// Number of requests to send
    #[clap(short = 'n', long)]
    requests: Option<u64>,

    /// Number of concurrent requests
    #[clap(short, long)]
    concurrency: Option<u64>,

    /// Run in interactive mode (guided configuration)
    #[clap(short, long)]
    interactive: bool,

    /// Skip confirmation and run immediately
    #[clap(short = 'y', long)]
    yes: bool,
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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    print_banner();

    // Determine if we should run in interactive mode
    let use_interactive = args.interactive || args.url.is_none();

    let config = if use_interactive {
        // Interactive mode - guide the user through configuration
        interactive::run_interactive_mode(args.url)?
    } else {
        // Quick mode - use CLI args with defaults
        let url = args.url.unwrap(); // Safe because we checked above
        let requests = args.requests.unwrap_or(100);
        let concurrency = args.concurrency.unwrap_or(10);

        client::LoadTestConfig::new(url, requests, concurrency)
    };

    // Show configuration summary
    interactive::display_config_summary(&config);

    // Confirm before running (unless --yes flag is set)
    if !args.yes {
        let theme = ColorfulTheme::default();
        let confirmed = Confirm::with_theme(&theme)
            .with_prompt("Start load test?")
            .default(true)
            .interact()?;

        if !confirmed {
            println!("{}", "Load test cancelled.".yellow());
            return Ok(());
        }
    }

    println!();
    println!("{}", "🚀 Starting load test...".yellow().bold());
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
