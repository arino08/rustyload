use crate::protocols::flashkv::{FlashKVCommand, FlashKVConfig};
use crate::protocols::http::{HttpConfig, HttpMethod};
use crate::protocols::{LoadTestConfig, Protocol};
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

    // Step 1: Select protocol
    let protocols = vec![
        "HTTP/HTTPS (Web APIs, REST endpoints)",
        "FlashKV (TCP key-value database)",
    ];

    // If URL is provided and starts with http, default to HTTP
    let default_protocol = if url.as_ref().map(|u| u.starts_with("http")).unwrap_or(false) {
        0
    } else {
        0
    };

    let protocol_index = Select::with_theme(&theme)
        .with_prompt("Select protocol")
        .items(&protocols)
        .default(default_protocol)
        .interact()?;

    let protocol = if protocol_index == 0 {
        Protocol::Http
    } else {
        Protocol::FlashKV
    };

    println!();

    match protocol {
        Protocol::Http => run_http_interactive_mode(url, &theme),
        Protocol::FlashKV => run_flashkv_interactive_mode(&theme),
    }
}

/// Run HTTP-specific interactive mode
fn run_http_interactive_mode(url: Option<String>, theme: &ColorfulTheme) -> Result<LoadTestConfig> {
    // Step 1: URL (use provided or ask)
    let url = match url {
        Some(u) if u.starts_with("http://") || u.starts_with("https://") => {
            println!("{} {}", "Target URL:".green(), u);
            u
        }
        _ => Input::with_theme(theme)
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
    let method_index = Select::with_theme(theme)
        .with_prompt("HTTP Method")
        .items(&methods)
        .default(0)
        .interact()?;

    let method = HttpMethod::from_str(methods[method_index]).unwrap();

    println!();

    // Step 3: Number of requests
    let num_requests: u64 = Input::with_theme(theme)
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
    let concurrency: u64 = Input::with_theme(theme)
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
    let timeout: u64 = Input::with_theme(theme)
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

    let optional_index = Select::with_theme(theme)
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
                let header: String = Input::with_theme(theme)
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
                if Confirm::with_theme(theme)
                    .with_prompt("Add a request body?")
                    .default(false)
                    .interact()?
                {
                    body = Some(
                        Input::with_theme(theme)
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
                    Input::with_theme(theme)
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
                let ct_index = Select::with_theme(theme)
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

        let header_selections = MultiSelect::with_theme(theme)
            .with_prompt("Add common headers? (Space to select, Enter to confirm)")
            .items(&common_headers)
            .interact()?;

        for selection in header_selections {
            match selection {
                0 => {
                    let token: String = Input::with_theme(theme)
                        .with_prompt("Bearer token")
                        .interact_text()?;
                    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                }
                1 => {
                    let header_name: String = Input::with_theme(theme)
                        .with_prompt("API Key header name")
                        .default("X-API-Key".to_string())
                        .interact_text()?;
                    let api_key: String = Input::with_theme(theme)
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

    // Build HTTP config
    let http_config = HttpConfig::new(url)
        .with_method(method)
        .with_headers(headers)
        .with_body(body);

    // Build and return config
    let config = LoadTestConfig {
        protocol: Protocol::Http,
        num_requests,
        concurrency,
        timeout_secs: timeout,
        http_config: Some(http_config),
        flashkv_config: None,
    };

    Ok(config)
}

/// Run FlashKV-specific interactive mode
fn run_flashkv_interactive_mode(theme: &ColorfulTheme) -> Result<LoadTestConfig> {
    println!("{}", "🗄️  FlashKV Load Test Configuration".magenta().bold());
    println!();

    // Step 1: Host
    let host: String = Input::with_theme(theme)
        .with_prompt("FlashKV server host")
        .default("localhost".to_string())
        .interact_text()?;

    println!();

    // Step 2: Port
    let port: u16 = Input::with_theme(theme)
        .with_prompt("FlashKV server port")
        .default(6379_u16)
        .interact_text()?;

    println!();

    // Step 3: Commands
    let command_options = vec![
        "PING - Check connectivity",
        "GET <key> - Read a value",
        "SET <key> <value> - Write a value",
        "GET + SET mixed workload",
        "INCR <key> - Increment counter",
        "Custom commands",
    ];

    let command_index = Select::with_theme(theme)
        .with_prompt("Select workload type")
        .items(&command_options)
        .default(0)
        .interact()?;

    let mut commands = Vec::new();

    match command_index {
        0 => {
            // PING
            commands.push(FlashKVCommand::Ping);
        }
        1 => {
            // GET
            let key: String = Input::with_theme(theme)
                .with_prompt("Key to GET (or use random keys below)")
                .default("testkey".to_string())
                .interact_text()?;
            commands.push(FlashKVCommand::Get { key });
        }
        2 => {
            // SET
            let key: String = Input::with_theme(theme)
                .with_prompt("Key to SET (or use random keys below)")
                .default("testkey".to_string())
                .interact_text()?;
            let value: String = Input::with_theme(theme)
                .with_prompt("Value to SET")
                .default("testvalue".to_string())
                .interact_text()?;
            commands.push(FlashKVCommand::Set { key, value });
        }
        3 => {
            // Mixed GET + SET
            println!();
            println!("{}", "Setting up mixed GET/SET workload...".dimmed());
            let key: String = Input::with_theme(theme)
                .with_prompt("Base key (or use random keys below)")
                .default("testkey".to_string())
                .interact_text()?;
            let value: String = Input::with_theme(theme)
                .with_prompt("Value for SET operations")
                .default("testvalue".to_string())
                .interact_text()?;

            commands.push(FlashKVCommand::Set {
                key: key.clone(),
                value,
            });
            commands.push(FlashKVCommand::Get { key });
        }
        4 => {
            // INCR
            let key: String = Input::with_theme(theme)
                .with_prompt("Counter key to INCR")
                .default("counter".to_string())
                .interact_text()?;
            commands.push(FlashKVCommand::Incr { key });
        }
        5 => {
            // Custom commands
            println!();
            println!(
                "{}",
                "Enter commands (one per line, empty line to finish):".dimmed()
            );
            println!(
                "{}",
                "Examples: GET key, SET key value, PING, DEL key".dimmed()
            );

            loop {
                let cmd_str: String = Input::with_theme(theme)
                    .with_prompt("Command")
                    .allow_empty(true)
                    .interact_text()?;

                if cmd_str.is_empty() {
                    break;
                }

                match FlashKVCommand::from_str(&cmd_str) {
                    Ok(cmd) => {
                        commands.push(cmd);
                        println!("{}", "✓ Command added".green());
                    }
                    Err(e) => {
                        println!("{} {}", "Invalid command:".red(), e);
                    }
                }
            }

            if commands.is_empty() {
                println!("{}", "No commands added, defaulting to PING".yellow());
                commands.push(FlashKVCommand::Ping);
            }
        }
        _ => {
            commands.push(FlashKVCommand::Ping);
        }
    }

    println!();

    // Step 4: Random keys?
    let use_random_keys = if command_index != 0 {
        // Only ask about random keys for commands that use keys
        Confirm::with_theme(theme)
            .with_prompt("Use random keys? (distributes load across key space)")
            .default(false)
            .interact()?
    } else {
        false
    };

    let mut key_prefix = "key".to_string();
    let mut key_range: u64 = 1000;

    if use_random_keys {
        println!();
        key_prefix = Input::with_theme(theme)
            .with_prompt("Key prefix")
            .default("key".to_string())
            .interact_text()?;

        key_range = Input::with_theme(theme)
            .with_prompt("Key range (0 to N-1)")
            .default(1000_u64)
            .interact_text()?;
    }

    println!();

    // Step 5: Number of requests
    let num_requests: u64 = Input::with_theme(theme)
        .with_prompt("Number of requests")
        .default(1000)
        .validate_with(|input: &u64| -> Result<(), &str> {
            if *input > 0 {
                Ok(())
            } else {
                Err("Must be at least 1 request")
            }
        })
        .interact_text()?;

    println!();

    // Step 6: Concurrency
    let concurrency: u64 = Input::with_theme(theme)
        .with_prompt("Concurrent connections")
        .default(50)
        .validate_with(|input: &u64| -> Result<(), &str> {
            if *input > 0 {
                Ok(())
            } else {
                Err("Must be at least 1")
            }
        })
        .interact_text()?;

    println!();

    // Step 7: Timeout
    let timeout: u64 = Input::with_theme(theme)
        .with_prompt("Timeout (seconds)")
        .default(10)
        .interact_text()?;

    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", "✅ Configuration complete!".green().bold());
    println!();

    // Build FlashKV config
    let flashkv_config = FlashKVConfig::new(host, port)
        .with_commands(commands)
        .with_random_keys(use_random_keys, key_prefix, key_range);

    // Build and return config
    let config = LoadTestConfig {
        protocol: Protocol::FlashKV,
        num_requests,
        concurrency,
        timeout_secs: timeout,
        http_config: None,
        flashkv_config: Some(flashkv_config),
    };

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
        "Protocol:".cyan(),
        config.protocol.display_name(),
        "│".dimmed()
    );

    println!(
        "{} {:<18} {:<28} {}",
        "│".dimmed(),
        "Target:".cyan(),
        truncate_string(&config.display_target(), 28),
        "│".dimmed()
    );

    match &config.protocol {
        Protocol::Http => {
            if let Some(http_config) = &config.http_config {
                println!(
                    "{} {:<18} {:<28} {}",
                    "│".dimmed(),
                    "Method:".cyan(),
                    format!("{:?}", http_config.method),
                    "│".dimmed()
                );
            }
        }
        Protocol::FlashKV => {
            if let Some(kv_config) = &config.flashkv_config {
                let commands_str = kv_config
                    .commands
                    .iter()
                    .map(|c| c.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                println!(
                    "{} {:<18} {:<28} {}",
                    "│".dimmed(),
                    "Commands:".magenta(),
                    truncate_string(&commands_str, 28),
                    "│".dimmed()
                );

                if kv_config.use_random_keys {
                    let random_info =
                        format!("{}:0-{}", kv_config.key_prefix, kv_config.key_range - 1);
                    println!(
                        "{} {:<18} {:<28} {}",
                        "│".dimmed(),
                        "Random Keys:".magenta(),
                        truncate_string(&random_info, 28),
                        "│".dimmed()
                    );
                }
            }
        }
    }

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

    // HTTP-specific details
    if let Some(http_config) = &config.http_config {
        if !http_config.headers.is_empty() {
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
            for (key, value) in &http_config.headers {
                let header_display = format!("  {}: {}", key, truncate_string(value, 20));
                println!(
                    "{} {:<47} {}",
                    "│".dimmed(),
                    header_display.dimmed(),
                    "│".dimmed()
                );
            }
        }

        if let Some(body) = &http_config.body {
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
