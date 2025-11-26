//! FlashKV TCP protocol implementation for load testing
//!
//! FlashKV is a Redis-like in-memory key-value database that communicates over TCP.
//! This module provides load testing capabilities for FlashKV servers.

use crate::protocols::{calculate_stats, LoadTestStats, RequestResult};
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use rand::Rng;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::{timeout, Duration};

/// Supported FlashKV commands
#[derive(Debug, Clone, PartialEq)]
pub enum FlashKVCommand {
    /// PING - Check server connectivity
    Ping,
    /// GET <key> - Retrieve a value
    Get { key: String },
    /// SET <key> <value> - Store a value
    Set { key: String, value: String },
    /// DEL <key> - Delete a key
    Del { key: String },
    /// INCR <key> - Increment a numeric value
    Incr { key: String },
    /// DECR <key> - Decrement a numeric value
    Decr { key: String },
    /// LPUSH <key> <value> - Push to list
    LPush { key: String, value: String },
    /// LPOP <key> - Pop from list
    LPop { key: String },
    /// EXISTS <key> - Check if key exists
    Exists { key: String },
    /// EXPIRE <key> <seconds> - Set key expiration
    Expire { key: String, seconds: u64 },
    /// TTL <key> - Get time to live
    Ttl { key: String },
    /// KEYS <pattern> - List keys matching pattern
    Keys { pattern: String },
    /// FLUSHDB - Clear all keys (use with caution!)
    FlushDb,
    /// Custom raw command
    Raw { command: String },
}

impl FlashKVCommand {
    /// Parse a command string into a FlashKVCommand
    pub fn from_str(s: &str) -> Result<Self, String> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        match parts[0].to_uppercase().as_str() {
            "PING" => Ok(FlashKVCommand::Ping),
            "GET" => {
                if parts.len() < 2 {
                    Err("GET requires a key".to_string())
                } else {
                    Ok(FlashKVCommand::Get {
                        key: parts[1].to_string(),
                    })
                }
            }
            "SET" => {
                if parts.len() < 3 {
                    Err("SET requires a key and value".to_string())
                } else {
                    Ok(FlashKVCommand::Set {
                        key: parts[1].to_string(),
                        value: parts[2..].join(" "),
                    })
                }
            }
            "DEL" | "DELETE" => {
                if parts.len() < 2 {
                    Err("DEL requires a key".to_string())
                } else {
                    Ok(FlashKVCommand::Del {
                        key: parts[1].to_string(),
                    })
                }
            }
            "INCR" => {
                if parts.len() < 2 {
                    Err("INCR requires a key".to_string())
                } else {
                    Ok(FlashKVCommand::Incr {
                        key: parts[1].to_string(),
                    })
                }
            }
            "DECR" => {
                if parts.len() < 2 {
                    Err("DECR requires a key".to_string())
                } else {
                    Ok(FlashKVCommand::Decr {
                        key: parts[1].to_string(),
                    })
                }
            }
            "LPUSH" => {
                if parts.len() < 3 {
                    Err("LPUSH requires a key and value".to_string())
                } else {
                    Ok(FlashKVCommand::LPush {
                        key: parts[1].to_string(),
                        value: parts[2..].join(" "),
                    })
                }
            }
            "LPOP" => {
                if parts.len() < 2 {
                    Err("LPOP requires a key".to_string())
                } else {
                    Ok(FlashKVCommand::LPop {
                        key: parts[1].to_string(),
                    })
                }
            }
            "EXISTS" => {
                if parts.len() < 2 {
                    Err("EXISTS requires a key".to_string())
                } else {
                    Ok(FlashKVCommand::Exists {
                        key: parts[1].to_string(),
                    })
                }
            }
            "EXPIRE" => {
                if parts.len() < 3 {
                    Err("EXPIRE requires a key and seconds".to_string())
                } else {
                    let seconds = parts[2]
                        .parse::<u64>()
                        .map_err(|_| "Invalid seconds value")?;
                    Ok(FlashKVCommand::Expire {
                        key: parts[1].to_string(),
                        seconds,
                    })
                }
            }
            "TTL" => {
                if parts.len() < 2 {
                    Err("TTL requires a key".to_string())
                } else {
                    Ok(FlashKVCommand::Ttl {
                        key: parts[1].to_string(),
                    })
                }
            }
            "KEYS" => {
                let pattern = if parts.len() < 2 {
                    "*".to_string()
                } else {
                    parts[1].to_string()
                };
                Ok(FlashKVCommand::Keys { pattern })
            }
            "FLUSHDB" => Ok(FlashKVCommand::FlushDb),
            _ => Ok(FlashKVCommand::Raw {
                command: s.to_string(),
            }),
        }
    }

    /// Convert the command to a wire format string
    pub fn to_wire_format(&self) -> String {
        match self {
            FlashKVCommand::Ping => "PING\r\n".to_string(),
            FlashKVCommand::Get { key } => format!("GET {}\r\n", key),
            FlashKVCommand::Set { key, value } => format!("SET {} {}\r\n", key, value),
            FlashKVCommand::Del { key } => format!("DEL {}\r\n", key),
            FlashKVCommand::Incr { key } => format!("INCR {}\r\n", key),
            FlashKVCommand::Decr { key } => format!("DECR {}\r\n", key),
            FlashKVCommand::LPush { key, value } => format!("LPUSH {} {}\r\n", key, value),
            FlashKVCommand::LPop { key } => format!("LPOP {}\r\n", key),
            FlashKVCommand::Exists { key } => format!("EXISTS {}\r\n", key),
            FlashKVCommand::Expire { key, seconds } => format!("EXPIRE {} {}\r\n", key, seconds),
            FlashKVCommand::Ttl { key } => format!("TTL {}\r\n", key),
            FlashKVCommand::Keys { pattern } => format!("KEYS {}\r\n", pattern),
            FlashKVCommand::FlushDb => "FLUSHDB\r\n".to_string(),
            FlashKVCommand::Raw { command } => {
                if command.ends_with("\r\n") {
                    command.clone()
                } else if command.ends_with('\n') {
                    format!("{}\r\n", command.trim_end())
                } else {
                    format!("{}\r\n", command)
                }
            }
        }
    }

    /// Create a command with a randomized key based on config
    pub fn with_random_key(&self, prefix: &str, range: u64) -> Self {
        let random_suffix: u64 = rand::rng().random_range(0..range);
        let random_key = format!("{}:{}", prefix, random_suffix);

        match self {
            FlashKVCommand::Get { .. } => FlashKVCommand::Get { key: random_key },
            FlashKVCommand::Set { value, .. } => FlashKVCommand::Set {
                key: random_key,
                value: value.clone(),
            },
            FlashKVCommand::Del { .. } => FlashKVCommand::Del { key: random_key },
            FlashKVCommand::Incr { .. } => FlashKVCommand::Incr { key: random_key },
            FlashKVCommand::Decr { .. } => FlashKVCommand::Decr { key: random_key },
            FlashKVCommand::LPush { value, .. } => FlashKVCommand::LPush {
                key: random_key,
                value: value.clone(),
            },
            FlashKVCommand::LPop { .. } => FlashKVCommand::LPop { key: random_key },
            FlashKVCommand::Exists { .. } => FlashKVCommand::Exists { key: random_key },
            FlashKVCommand::Expire { seconds, .. } => FlashKVCommand::Expire {
                key: random_key,
                seconds: *seconds,
            },
            FlashKVCommand::Ttl { .. } => FlashKVCommand::Ttl { key: random_key },
            // Commands that don't use keys
            _ => self.clone(),
        }
    }

    /// Get the display name of the command
    pub fn display_name(&self) -> &'static str {
        match self {
            FlashKVCommand::Ping => "PING",
            FlashKVCommand::Get { .. } => "GET",
            FlashKVCommand::Set { .. } => "SET",
            FlashKVCommand::Del { .. } => "DEL",
            FlashKVCommand::Incr { .. } => "INCR",
            FlashKVCommand::Decr { .. } => "DECR",
            FlashKVCommand::LPush { .. } => "LPUSH",
            FlashKVCommand::LPop { .. } => "LPOP",
            FlashKVCommand::Exists { .. } => "EXISTS",
            FlashKVCommand::Expire { .. } => "EXPIRE",
            FlashKVCommand::Ttl { .. } => "TTL",
            FlashKVCommand::Keys { .. } => "KEYS",
            FlashKVCommand::FlushDb => "FLUSHDB",
            FlashKVCommand::Raw { .. } => "RAW",
        }
    }
}

/// FlashKV-specific configuration
#[derive(Debug, Clone)]
pub struct FlashKVConfig {
    /// Server hostname
    pub host: String,
    /// Server port
    pub port: u16,
    /// Commands to execute (will be cycled through)
    pub commands: Vec<FlashKVCommand>,
    /// Whether to use random keys for each request
    pub use_random_keys: bool,
    /// Prefix for random keys
    pub key_prefix: String,
    /// Range for random key generation (0 to key_range-1)
    pub key_range: u64,
}

impl FlashKVConfig {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            commands: vec![FlashKVCommand::Ping],
            use_random_keys: false,
            key_prefix: "key".to_string(),
            key_range: 1000,
        }
    }

    pub fn with_commands(mut self, commands: Vec<FlashKVCommand>) -> Self {
        self.commands = commands;
        self
    }

    pub fn with_random_keys(mut self, use_random: bool, prefix: String, range: u64) -> Self {
        self.use_random_keys = use_random;
        self.key_prefix = prefix;
        self.key_range = range;
        self
    }

    /// Get the server address
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Response status codes for FlashKV
pub mod status {
    /// Successful operation
    pub const OK: u16 = 200;
    /// Key not found (still successful operation)
    pub const NOT_FOUND: u16 = 404;
    /// Server error
    pub const ERROR: u16 = 500;
    /// Connection error
    pub const CONNECTION_ERROR: u16 = 503;
    /// Timeout
    pub const TIMEOUT: u16 = 504;
}

/// Fire a single FlashKV request
pub async fn fire_single_request(
    config: &FlashKVConfig,
    command_index: usize,
    timeout_secs: u64,
) -> RequestResult {
    let start = Instant::now();

    // Get the command to execute (cycle through commands)
    let base_command = &config.commands[command_index % config.commands.len()];

    // Apply random key if configured
    let command = if config.use_random_keys {
        base_command.with_random_key(&config.key_prefix, config.key_range)
    } else {
        base_command.clone()
    };

    let wire_command = command.to_wire_format();

    // Try to connect and send the command
    match timeout(
        Duration::from_secs(timeout_secs),
        execute_command(&config.address(), &wire_command),
    )
    .await
    {
        Ok(Ok((response, is_error))) => {
            let duration = start.elapsed().as_millis();
            let (status, success) = if is_error {
                (status::ERROR, false)
            } else if response.to_uppercase().contains("NIL")
                || response.to_uppercase().contains("NOT FOUND")
                || response.to_uppercase().contains("(nil)")
            {
                // Key not found is still a successful operation
                (status::NOT_FOUND, true)
            } else {
                (status::OK, true)
            };

            RequestResult {
                duration,
                status,
                success,
                error: if is_error { Some(response) } else { None },
            }
        }
        Ok(Err(e)) => {
            let duration = start.elapsed().as_millis();
            RequestResult {
                duration,
                status: status::CONNECTION_ERROR,
                success: false,
                error: Some(e.to_string()),
            }
        }
        Err(_) => {
            let duration = start.elapsed().as_millis();
            RequestResult {
                duration,
                status: status::TIMEOUT,
                success: false,
                error: Some("Request timed out".to_string()),
            }
        }
    }
}

/// Execute a command on the FlashKV server
async fn execute_command(address: &str, command: &str) -> Result<(String, bool)> {
    // Connect to the server
    let stream = TcpStream::connect(address)
        .await
        .context("Failed to connect to FlashKV server")?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Send the command
    writer
        .write_all(command.as_bytes())
        .await
        .context("Failed to send command")?;
    writer.flush().await.context("Failed to flush")?;

    // Read the response (assuming line-based protocol)
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .await
        .context("Failed to read response")?;

    let response = response.trim().to_string();

    // Check if response indicates an error
    let is_error = response.starts_with("-ERR")
        || response.starts_with("ERROR")
        || response.starts_with("-")
        || response.to_uppercase().starts_with("ERR");

    Ok((response, is_error))
}

/// Run a FlashKV load test with the given configuration
pub async fn run_load_test(
    config: &FlashKVConfig,
    num_requests: u64,
    concurrency: u64,
    timeout_secs: u64,
) -> Result<LoadTestStats> {
    let config = Arc::new(config.clone());
    let semaphore = Arc::new(Semaphore::new(concurrency as usize));

    // Create progress bar
    let pb = ProgressBar::new(num_requests);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.magenta/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );

    let commands_desc = config
        .commands
        .iter()
        .map(|c| c.display_name())
        .collect::<Vec<_>>()
        .join(", ");
    pb.set_message(format!("Sending FlashKV commands: {}...", commands_desc));

    let overall_start = Instant::now();

    // Spawn all tasks
    let mut handles = Vec::with_capacity(num_requests as usize);

    for i in 0..num_requests {
        let config = Arc::clone(&config);
        let semaphore = Arc::clone(&semaphore);
        let pb = pb.clone();

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();
            let result = fire_single_request(&config, i as usize, timeout_secs).await;
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
    fn test_command_from_str_ping() {
        let cmd = FlashKVCommand::from_str("PING").unwrap();
        assert_eq!(cmd, FlashKVCommand::Ping);
    }

    #[test]
    fn test_command_from_str_get() {
        let cmd = FlashKVCommand::from_str("GET mykey").unwrap();
        assert_eq!(
            cmd,
            FlashKVCommand::Get {
                key: "mykey".to_string()
            }
        );
    }

    #[test]
    fn test_command_from_str_set() {
        let cmd = FlashKVCommand::from_str("SET mykey myvalue").unwrap();
        assert_eq!(
            cmd,
            FlashKVCommand::Set {
                key: "mykey".to_string(),
                value: "myvalue".to_string()
            }
        );
    }

    #[test]
    fn test_command_from_str_set_with_spaces() {
        let cmd = FlashKVCommand::from_str("SET mykey hello world").unwrap();
        assert_eq!(
            cmd,
            FlashKVCommand::Set {
                key: "mykey".to_string(),
                value: "hello world".to_string()
            }
        );
    }

    #[test]
    fn test_command_from_str_del() {
        let cmd = FlashKVCommand::from_str("DEL mykey").unwrap();
        assert_eq!(
            cmd,
            FlashKVCommand::Del {
                key: "mykey".to_string()
            }
        );
    }

    #[test]
    fn test_command_from_str_expire() {
        let cmd = FlashKVCommand::from_str("EXPIRE mykey 3600").unwrap();
        assert_eq!(
            cmd,
            FlashKVCommand::Expire {
                key: "mykey".to_string(),
                seconds: 3600
            }
        );
    }

    #[test]
    fn test_command_from_str_case_insensitive() {
        let cmd = FlashKVCommand::from_str("ping").unwrap();
        assert_eq!(cmd, FlashKVCommand::Ping);

        let cmd = FlashKVCommand::from_str("get KEY").unwrap();
        assert_eq!(
            cmd,
            FlashKVCommand::Get {
                key: "KEY".to_string()
            }
        );
    }

    #[test]
    fn test_command_from_str_raw() {
        let cmd = FlashKVCommand::from_str("CUSTOMCMD arg1 arg2").unwrap();
        assert_eq!(
            cmd,
            FlashKVCommand::Raw {
                command: "CUSTOMCMD arg1 arg2".to_string()
            }
        );
    }

    #[test]
    fn test_command_missing_args() {
        assert!(FlashKVCommand::from_str("GET").is_err());
        assert!(FlashKVCommand::from_str("SET key").is_err());
    }

    #[test]
    fn test_wire_format() {
        assert_eq!(FlashKVCommand::Ping.to_wire_format(), "PING\r\n");
        assert_eq!(
            FlashKVCommand::Get {
                key: "test".to_string()
            }
            .to_wire_format(),
            "GET test\r\n"
        );
        assert_eq!(
            FlashKVCommand::Set {
                key: "test".to_string(),
                value: "value".to_string()
            }
            .to_wire_format(),
            "SET test value\r\n"
        );
    }

    #[test]
    fn test_config_builder() {
        let config = FlashKVConfig::new("localhost".to_string(), 6379)
            .with_commands(vec![
                FlashKVCommand::Ping,
                FlashKVCommand::Get {
                    key: "test".to_string(),
                },
            ])
            .with_random_keys(true, "prefix".to_string(), 500);

        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 6379);
        assert_eq!(config.commands.len(), 2);
        assert!(config.use_random_keys);
        assert_eq!(config.key_prefix, "prefix");
        assert_eq!(config.key_range, 500);
    }

    #[test]
    fn test_address() {
        let config = FlashKVConfig::new("127.0.0.1".to_string(), 6379);
        assert_eq!(config.address(), "127.0.0.1:6379");
    }

    #[test]
    fn test_with_random_key() {
        let cmd = FlashKVCommand::Get {
            key: "original".to_string(),
        };
        let random_cmd = cmd.with_random_key("prefix", 100);

        if let FlashKVCommand::Get { key } = random_cmd {
            assert!(key.starts_with("prefix:"));
        } else {
            panic!("Expected Get command");
        }
    }

    #[test]
    fn test_ping_no_random_key() {
        let cmd = FlashKVCommand::Ping;
        let random_cmd = cmd.with_random_key("prefix", 100);
        assert_eq!(random_cmd, FlashKVCommand::Ping);
    }
}
