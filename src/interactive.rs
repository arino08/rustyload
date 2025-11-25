use crate::client::{HttpMethod, LoadTestConfig};
use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use std::collections::HashMap;

/// Runs the interactive TUI to gather configuration from the user
pub fn run_interactive_mode(url: Option<String>) -> Result<LoadTestConfig> {
    println!();
    println!(
        "{}",
        "🚀 Interactive Mode - Let's configure your load test!"
            .cyan()
            .bold()
    );
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let theme = ColorfulTheme::default();

    // Step 1: URL (use provided or ask)
    let url = match url {
        Some(u) => {
            println!("{} {}", "Target URL:".green(), u);
            u
        }
        None => Input::with_theme(&theme)
            .with_prompt("Target URL")
            .with_initial_text("https://")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.starts_with("http://") || input.starts_with("https://") {
                    Ok(())
                } else {
                    Err("URL must start with http:// or https://")
                }
            })
            .interact_text()?,
    };

    println!();

    // Step 2: HTTP Method
    let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"];
    let method_index = Select::with_theme(&theme)
        .with_prompt("HTTP Method")
        .items(&methods)
        .default(0)
        .interact()?;

    let method = HttpMethod::from_str(methods[method_index]).unwrap();

    println!();

    // Step 3: Number of requests
    let num_requests: u64 = Input::with_theme(&theme)
        .with_prompt("Number of requests")
        .default(100)
        .validate_with(|input: &u64| -> Result<(), &str> {
            if *input > 0 {
                Ok(())
            } else {
                Err("Must be at least 1 request")
            }
        })
        .interact_text()?;

    println!();

    // Step 4: Concurrency
    let concurrency: u64 = Input::with_theme(&theme)
        .with_prompt("Concurrent requests")
        .default(10)
        .validate_with(|input: &u64| -> Result<(), &str> {
            if *input > 0 {
                Ok(())
            } else {
                Err("Must be at least 1")
            }
        })
        .interact_text()?;

    println!();

    // Step 5: Timeout
    let timeout: u64 = Input::with_theme(&theme)
        .with_prompt("Timeout (seconds)")
        .default(30)
        .interact_text()?;

    println!();

    // Step 6: Optional features
    let optional_features = vec![
        "Add custom headers",
        "Add request body",
        "Skip (use defaults)",
    ];

    let optional_index = Select::with_theme(&theme)
        .with_prompt("Additional options")
        .items(&optional_features)
        .default(2)
        .interact()?;

    let mut headers: HashMap<String, String> = HashMap::new();
    let mut body: Option<String> = None;

    match optional_index {
        0 => {
            // Add custom headers
            println!();
            println!("{}", "Enter headers (empty line to finish):".dimmed());

            loop {
                let header: String = Input::with_theme(&theme)
                    .with_prompt("Header (Key: Value)")
                    .allow_empty(true)
                    .interact_text()?;

                if header.is_empty() {
                    break;
                }

                if let Some((key, value)) = parse_header_input(&header) {
                    headers.insert(key, value);
                } else {
                    println!("{}", "Invalid format. Use 'Key: Value'".red());
                }
            }

            // Ask if they also want to add a body
            if matches!(
                method,
                HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
            ) {
                println!();
                if Confirm::with_theme(&theme)
                    .with_prompt("Add a request body?")
                    .default(false)
                    .interact()?
                {
                    body = Some(
                        Input::with_theme(&theme)
                            .with_prompt("Request body")
                            .interact_text()?,
                    );
                }
            }
        }
        1 => {
            // Add request body
            if matches!(
                method,
                HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
            ) {
                println!();
                body = Some(
                    Input::with_theme(&theme)
                        .with_prompt("Request body")
                        .interact_text()?,
                );

                // Suggest Content-Type header
                println!();
                let content_types = vec![
                    "application/json",
                    "application/x-www-form-urlencoded",
                    "text/plain",
                    "None (skip)",
                ];
                let ct_index = Select::with_theme(&theme)
                    .with_prompt("Content-Type")
                    .items(&content_types)
                    .default(0)
                    .interact()?;

                if ct_index < 3 {
                    headers.insert(
                        "Content-Type".to_string(),
                        content_types[ct_index].to_string(),
                    );
                }
            } else {
                println!(
                    "{}",
                    "Note: Request body is only used with POST, PUT, or PATCH methods.".yellow()
                );
            }
        }
        _ => {
            // Skip - use defaults
        }
    }

    println!();

    // Step 7: Add common headers?
    if headers.is_empty() {
        let common_headers = vec![
            "Authorization (Bearer token)",
            "API Key header",
            "Accept: application/json",
            "No additional headers",
        ];

        let header_selections = MultiSelect::with_theme(&theme)
            .with_prompt("Add common headers? (Space to select, Enter to confirm)")
            .items(&common_headers)
            .interact()?;

        for selection in header_selections {
            match selection {
                0 => {
                    let token: String = Input::with_theme(&theme)
                        .with_prompt("Bearer token")
                        .interact_text()?;
                    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                }
                1 => {
                    let header_name: String = Input::with_theme(&theme)
                        .with_prompt("API Key header name")
                        .default("X-API-Key".to_string())
                        .interact_text()?;
                    let api_key: String = Input::with_theme(&theme)
                        .with_prompt("API Key value")
                        .interact_text()?;
                    headers.insert(header_name, api_key);
                }
                2 => {
                    headers.insert("Accept".to_string(), "application/json".to_string());
                }
                _ => {}
            }
        }
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", "✅ Configuration complete!".green().bold());
    println!();

    // Build and return config
    let config = LoadTestConfig::new(url, num_requests, concurrency)
        .with_method(method)
        .with_headers(headers)
        .with_body(body)
        .with_timeout(timeout);

    Ok(config)
}

/// Parse a header input string into key-value pair
fn parse_header_input(input: &str) -> Option<(String, String)> {
    // Try ": " first
    if let Some((key, value)) = input.split_once(": ") {
        return Some((key.trim().to_string(), value.trim().to_string()));
    }
    // Try ":" without space
    if let Some((key, value)) = input.split_once(':') {
        return Some((key.trim().to_string(), value.trim().to_string()));
    }
    // Try "="
    if let Some((key, value)) = input.split_once('=') {
        return Some((key.trim().to_string(), value.trim().to_string()));
    }
    None
}

/// Display a summary of the configuration before running
pub fn display_config_summary(config: &LoadTestConfig) {
    println!(
        "{}",
        "┌─────────────────────────────────────────────────┐".dimmed()
    );
    println!(
        "{} {:<47} {}",
        "│".dimmed(),
        "📋 Configuration Summary".white().bold(),
        "│".dimmed()
    );
    println!(
        "{}",
        "├─────────────────────────────────────────────────┤".dimmed()
    );

    println!(
        "{} {:<18} {:<28} {}",
        "│".dimmed(),
        "URL:".cyan(),
        truncate_string(&config.url, 28),
        "│".dimmed()
    );

    println!(
        "{} {:<18} {:<28} {}",
        "│".dimmed(),
        "Method:".cyan(),
        format!("{:?}", config.method),
        "│".dimmed()
    );

    println!(
        "{} {:<18} {:<28} {}",
        "│".dimmed(),
        "Requests:".cyan(),
        config.num_requests,
        "│".dimmed()
    );

    println!(
        "{} {:<18} {:<28} {}",
        "│".dimmed(),
        "Concurrency:".cyan(),
        config.concurrency,
        "│".dimmed()
    );

    println!(
        "{} {:<18} {:<28} {}",
        "│".dimmed(),
        "Timeout:".cyan(),
        format!("{}s", config.timeout_secs),
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
            "Headers:".cyan(),
            "│".dimmed()
        );
        for (key, value) in &config.headers {
            let header_display = format!("  {}: {}", key, truncate_string(value, 20));
            println!(
                "{} {:<47} {}",
                "│".dimmed(),
                header_display.dimmed(),
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
            "{} {:<18} {:<28} {}",
            "│".dimmed(),
            "Body:".cyan(),
            truncate_string(body, 28),
            "│".dimmed()
        );
    }

    println!(
        "{}",
        "└─────────────────────────────────────────────────┘".dimmed()
    );
    println!();
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
