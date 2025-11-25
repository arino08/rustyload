# 🦀 RustyLoad Technical Guide

A comprehensive, beginner-friendly guide explaining every aspect of how RustyLoad works internally. This document assumes you know basic Rust syntax (variables, functions, structs) but explains all the advanced concepts in detail.

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Project Structure](#project-structure)
3. [Cargo.toml Explained](#cargotoml-explained)
4. [The Client Module (client.rs)](#the-client-module-clientrs)
   - [Imports](#imports)
   - [HttpMethod Enum](#httpmethod-enum)
   - [LoadTestConfig (Builder Pattern)](#loadtestconfig-builder-pattern)
   - [RequestResult Struct](#requestresult-struct)
   - [LoadTestStats Struct](#loadteststats-struct)
   - [fire_single_request Function](#fire_single_request-function)
   - [run_load_test Function](#run_load_test-function)
   - [Statistics Calculation](#statistics-calculation)
   - [Percentile Calculation](#percentile-calculation)
5. [The Interactive Module (interactive.rs)](#the-interactive-module-interactivers)
   - [Dialoguer Prompts](#dialoguer-prompts)
   - [Input Validation](#input-validation)
   - [Configuration Summary](#configuration-summary)
6. [The Main Module (main.rs)](#the-main-module-mainrs)
   - [CLI Argument Parsing](#cli-argument-parsing)
   - [Mode Selection](#mode-selection)
   - [Results Display](#results-display)
7. [Key Rust Concepts Used](#key-rust-concepts-used)
8. [Testing](#testing)
9. [GitHub Actions CI/CD](#github-actions-cicd)

---

## Project Overview

RustyLoad is an HTTP load testing tool that:
1. Sends multiple concurrent HTTP requests to a target URL
2. Measures response times (latency)
3. Calculates statistics including percentiles (p50, p95, p99)
4. Displays results in a beautiful terminal UI

### Why These Technologies?

| Technology | Why We Use It |
|------------|---------------|
| **Rust** | Memory safety, high performance, no garbage collector |
| **Tokio** | Async runtime - handles thousands of concurrent requests efficiently |
| **reqwest** | Popular, well-maintained HTTP client for Rust |
| **clap** | Industry-standard CLI argument parsing |
| **dialoguer** | Beautiful interactive terminal prompts |
| **indicatif** | Progress bars and spinners |
| **colored** | Terminal colors for better UX |

---

## Project Structure

```
rustyload/
├── .github/
│   └── workflows/
│       ├── ci.yml              # GitHub Actions CI/CD pipeline
│       └── release.yml         # Automated release builds
├── docs/
│   └── TECHNICAL_GUIDE.md      # This file!
├── src/
│   ├── main.rs                 # Entry point, CLI, orchestration
│   ├── client.rs               # HTTP client, load testing logic, statistics
│   └── interactive.rs          # Interactive TUI prompts
├── Cargo.toml                  # Dependencies and project metadata
├── Cargo.lock                  # Locked dependency versions
├── README.md                   # User-facing documentation
├── LICENSE                     # MIT License
└── .gitignore                  # Git ignore rules
```

### How the Modules Connect

```
┌─────────────────────────────────────────────────────────────┐
│                         main.rs                              │
│  - Parses CLI arguments                                      │
│  - Decides: interactive or quick mode?                       │
│  - Displays results                                          │
└─────────────────────────────────────────────────────────────┘
                    │                    │
                    ▼                    ▼
┌─────────────────────────────┐  ┌─────────────────────────────┐
│      interactive.rs          │  │        client.rs             │
│  - Guided prompts            │  │  - HTTP requests             │
│  - Input validation          │  │  - Concurrency control       │
│  - Config building           │  │  - Statistics calculation    │
└─────────────────────────────┘  └─────────────────────────────┘
                    │                    │
                    └──────────┬─────────┘
                               ▼
                    ┌─────────────────────┐
                    │   LoadTestConfig    │
                    │  (shared data type) │
                    └─────────────────────┘
```

---

## Cargo.toml Explained

```toml
[package]
name = "rustyload"
version = "0.1.0"
edition = "2021"
authors = ["Ariz"]
description = "A blazingly fast, concurrent HTTP load testing tool built in Rust"
license = "MIT"
repository = "https://github.com/yourusername/rustyload"
keywords = ["http", "load-testing", "benchmark", "performance", "cli"]
categories = ["command-line-utilities", "development-tools::profiling"]
readme = "README.md"
```

### Package Metadata Explained

| Field | Purpose |
|-------|---------|
| `name` | Crate name (lowercase convention) |
| `version` | Semantic versioning: MAJOR.MINOR.PATCH |
| `edition` | Rust edition - "2021" is current stable |
| `authors` | Your name (shows in `cargo metadata`) |
| `description` | Short description for crates.io |
| `license` | License identifier (MIT, Apache-2.0, etc.) |
| `repository` | Link to source code |
| `keywords` | Search terms for crates.io (max 5) |
| `categories` | Predefined categories on crates.io |
| `readme` | Path to README file |

### Dependencies Explained

```toml
[dependencies]
anyhow = "1.0"           # Easy error handling
clap = { version = "4.5", features = ["derive"] }  # CLI parsing with macros
colored = "3.0"          # Terminal colors
dialoguer = { version = "0.11", features = ["fuzzy-select"] }  # Interactive prompts
indicatif = "0.17"       # Progress bars
reqwest = { version = "0.12", features = ["json"] }  # HTTP client
tokio = { version = "1.48", features = ["full"] }    # Async runtime
```

#### What Each Dependency Does:

| Crate | Purpose | Example Usage |
|-------|---------|---------------|
| `anyhow` | Ergonomic error handling | `Result<T>` instead of `Result<T, E>` |
| `clap` | CLI argument parsing | `#[derive(Parser)]` |
| `colored` | Terminal colors | `"text".green().bold()` |
| `dialoguer` | Interactive prompts | `Select::new().items(&["a", "b"])` |
| `indicatif` | Progress bars | `ProgressBar::new(100)` |
| `reqwest` | HTTP client | `client.get(url).send().await` |
| `tokio` | Async runtime | `#[tokio::main]`, `async/await` |

### Release Profile

```toml
[profile.release]
opt-level = 3      # Maximum optimization (0-3)
lto = true         # Link-Time Optimization
codegen-units = 1  # Single compilation unit (better optimization)
strip = true       # Remove debug symbols (smaller binary)
```

**Result:** Release binary is ~70% smaller and significantly faster!

---

## The Client Module (client.rs)

This module handles all HTTP logic, concurrency, and statistics.

### Imports

```rust
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Method};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
```

#### Import Explanations:

| Import | What It Does |
|--------|--------------|
| `anyhow::{Context, Result}` | Error handling with context messages |
| `indicatif::{ProgressBar, ProgressStyle}` | Terminal progress bar |
| `reqwest::{Client, Method}` | HTTP client and method enum |
| `std::collections::HashMap` | Key-value storage for headers |
| `std::sync::Arc` | Atomic Reference Counter - shared ownership |
| `std::time::Instant` | High-precision timing |
| `tokio::sync::Semaphore` | Concurrency limiter |

---

### HttpMethod Enum

```rust
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
```

#### What is an Enum?

An enum (enumeration) represents a type that can be ONE of several variants:

```rust
let method = HttpMethod::GET;    // Valid
let method = HttpMethod::POST;   // Valid
let method = HttpMethod::BANANA; // Compile error! Not a variant
```

#### Derive Macros Explained:

| Derive | What It Does |
|--------|--------------|
| `Debug` | Allows printing with `{:?}` format |
| `Clone` | Allows `.clone()` to make copies |
| `Default` | Provides `HttpMethod::default()` → returns `GET` |

#### The `#[default]` Attribute:

```rust
#[default]
GET,  // This variant is returned by HttpMethod::default()
```

#### Implementation Block:

```rust
impl HttpMethod {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::GET),
            "POST" => Ok(HttpMethod::POST),
            // ... etc
            _ => Err(format!("Unsupported HTTP method: {}", s)),
        }
    }

    fn to_reqwest_method(&self) -> Method {
        match self {
            HttpMethod::GET => Method::GET,
            // ... etc
        }
    }
}
```

**`impl HttpMethod`** - Adds methods to the `HttpMethod` type.

**`pub fn from_str`** - Public function to parse strings like "GET" or "post" into our enum.

**`fn to_reqwest_method`** - Private function (no `pub`) to convert to reqwest's `Method` type.

**Why have our own enum?**
1. Control which methods we support
2. Add custom parsing logic
3. Easier testing
4. Decouples from reqwest (easier to switch libraries later)

---

### LoadTestConfig (Builder Pattern)

```rust
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
```

#### Fields Explained:

| Field | Type | Purpose |
|-------|------|---------|
| `url` | `String` | Target URL to test |
| `method` | `HttpMethod` | HTTP method (GET, POST, etc.) |
| `headers` | `HashMap<String, String>` | Custom headers (key-value pairs) |
| `body` | `Option<String>` | Optional request body |
| `num_requests` | `u64` | Total number of requests |
| `concurrency` | `u64` | Max concurrent requests |
| `timeout_secs` | `u64` | Request timeout in seconds |

#### What is `Option<T>`?

`Option` represents "maybe a value" - it's Rust's way of handling null safely:

```rust
let body: Option<String> = Some("hello".to_string()); // Has value
let body: Option<String> = None;                       // No value

// To use the value, you must handle both cases:
match body {
    Some(content) => println!("Body: {}", content),
    None => println!("No body"),
}

// Or use if let:
if let Some(content) = body {
    println!("Body: {}", content);
}
```

#### What is `HashMap<K, V>`?

A HashMap stores key-value pairs, like a dictionary in Python:

```rust
let mut headers = HashMap::new();
headers.insert("Content-Type".to_string(), "application/json".to_string());
headers.insert("Authorization".to_string(), "Bearer token123".to_string());

// Access:
if let Some(value) = headers.get("Content-Type") {
    println!("Content-Type is: {}", value);
}
```

#### The Builder Pattern:

```rust
impl LoadTestConfig {
    // Constructor - creates with defaults
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

    // Builder methods - modify and return self
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
```

**Why the Builder Pattern?**

Instead of this ugly constructor:
```rust
// Hard to read, easy to mess up order
let config = LoadTestConfig {
    url: "http://example.com".to_string(),
    method: HttpMethod::POST,
    headers: my_headers,
    body: Some("{}".to_string()),
    num_requests: 100,
    concurrency: 10,
    timeout_secs: 30,
};
```

You can write:
```rust
// Clear, readable, order doesn't matter
let config = LoadTestConfig::new("http://example.com".to_string(), 100, 10)
    .with_method(HttpMethod::POST)
    .with_headers(my_headers)
    .with_body(Some("{}".to_string()))
    .with_timeout(60);
```

**How `with_*` Methods Work:**

```rust
pub fn with_method(mut self, method: HttpMethod) -> Self {
    self.method = method;  // Modify the field
    self                   // Return self for chaining
}
```

| Part | Meaning |
|------|---------|
| `mut self` | Takes ownership AND makes it mutable |
| `self.method = method` | Modifies the field |
| `self` | Returns the modified struct |
| `-> Self` | Return type is the same struct |

**Why `mut self` not `&mut self`?**
- `&mut self` borrows, so you'd return a reference
- `mut self` takes ownership, modifies, returns ownership
- Enables chaining: `.with_a().with_b().with_c()`

---

### RequestResult Struct

```rust
#[derive(Debug, Clone)]
pub struct RequestResult {
    pub duration: u128,
    pub status: u16,
    pub success: bool,
    pub error: Option<String>,
}
```

Stores the result of a single HTTP request:

| Field | Type | Purpose |
|-------|------|---------|
| `duration` | `u128` | Response time in milliseconds |
| `status` | `u16` | HTTP status code (200, 404, etc.) |
| `success` | `bool` | Was it a 2xx response? |
| `error` | `Option<String>` | Error message if request failed |

**Why `u128` for duration?**
- `u128` can hold values up to 340 undecillion
- Overkill? Yes. But `.as_millis()` returns `u128`
- No casting needed, no overflow possible

---

### LoadTestStats Struct

```rust
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
```

Aggregated statistics after all requests complete:

| Field | Purpose |
|-------|---------|
| `total_requests` | How many requests we sent |
| `successful_requests` | How many got 2xx responses |
| `failed_requests` | How many failed |
| `total_duration` | Total test time in ms |
| `min_latency` | Fastest request |
| `max_latency` | Slowest request |
| `avg_latency` | Average response time |
| `p50` | 50th percentile (median) |
| `p95` | 95th percentile |
| `p99` | 99th percentile |
| `requests_per_second` | Throughput |

---

### fire_single_request Function

```rust
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
```

#### Function Signature Breakdown:

```rust
pub async fn fire_single_request(
    client: &Client,           // Borrow the HTTP client
    url: &str,                 // Borrow the URL string
    method: &HttpMethod,       // Borrow the method
    headers: &HashMap<...>,    // Borrow the headers
    body: &Option<String>,     // Borrow the optional body
) -> RequestResult {           // Return owned result
```

**Why borrow (`&`) everything?**
- We don't need ownership, just need to read the values
- Avoids cloning (more efficient)
- Caller keeps ownership

**`async fn`** - This function can be paused and resumed. When it hits `.await`, other code can run.

#### Timing with Instant:

```rust
let start = Instant::now();           // Start stopwatch
// ... do stuff ...
let duration = start.elapsed().as_millis();  // Get elapsed time
```

`Instant` provides high-precision timing, much better than wall-clock time.

#### Building the Request:

```rust
let mut request_builder = client.request(method.to_reqwest_method(), url);
```

- `client.request(Method, url)` - Creates a request builder for ANY method
- `mut` because we'll modify it by adding headers/body

#### Adding Headers:

```rust
for (key, value) in headers {
    request_builder = request_builder.header(key, value);
}
```

- Loop through each key-value pair in the HashMap
- `.header()` returns a new builder, so we reassign

#### Adding Body:

```rust
if let Some(body_content) = body {
    request_builder = request_builder.body(body_content.clone());
}
```

- `if let Some(x) = option` - Pattern matching for Option
- Only adds body if it exists (is `Some`)
- `.clone()` because `.body()` takes ownership

#### Sending and Handling Response:

```rust
match request_builder.send().await {
    Ok(response) => {
        // Success path
        RequestResult { duration, status, success: true, error: None }
    }
    Err(e) => {
        // Error path
        RequestResult { duration, status: 0, success: false, error: Some(e.to_string()) }
    }
}
```

- `.send().await` - Send request and wait for response
- `match` handles both success and failure
- No `return` needed - last expression is returned

---

### run_load_test Function

This is the main orchestration function:

```rust
pub async fn run_load_test(config: LoadTestConfig) -> Result<LoadTestStats> {
    // 1. Build HTTP client
    let client = Client::builder()
        .user_agent("rustyload/0.1")
        .timeout(std::time::Duration::from_secs(config.timeout_secs))
        .build()
        .context("Failed to build HTTP client")?;

    // 2. Wrap shared data in Arc
    let client = Arc::new(client);
    let semaphore = Arc::new(Semaphore::new(config.concurrency as usize));
    let method = Arc::new(config.method);
    let headers = Arc::new(config.headers);
    let body = Arc::new(config.body);

    // 3. Create progress bar
    let pb = ProgressBar::new(config.num_requests);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%) {msg}")
            .unwrap()
            .progress_chars("█▓▒░  "),
    );

    // 4. Spawn all tasks
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

    // 5. Collect results
    let mut results = Vec::with_capacity(config.num_requests as usize);
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    // 6. Calculate statistics
    let stats = calculate_stats(&results, total_duration);

    Ok(stats)
}
```

#### Step 1: Building the HTTP Client

```rust
let client = Client::builder()
    .user_agent("rustyload/0.1")
    .timeout(std::time::Duration::from_secs(config.timeout_secs))
    .build()
    .context("Failed to build HTTP client")?;
```

- `.user_agent()` - Sets the User-Agent header for all requests
- `.timeout()` - Sets request timeout
- `.build()` - Creates the client
- `.context("...")` - If error, add this message
- `?` - Return early if error

#### Step 2: Understanding Arc (Atomic Reference Counter)

```rust
let client = Arc::new(client);
let semaphore = Arc::new(Semaphore::new(config.concurrency as usize));
```

**What is Arc?**

`Arc` lets multiple owners share the same data safely across threads/tasks:

```
Without Arc:
┌──────────┐
│  client  │──────► Only ONE owner
└──────────┘

With Arc:
┌──────────┐
│   Arc    │◄────── Task 1 has a "ticket"
│  ┌────┐  │◄────── Task 2 has a "ticket"
│  │data│  │◄────── Task 3 has a "ticket"
│  └────┘  │
└──────────┘
   (reference count = 3)
```

When all "tickets" (Arc clones) are dropped, the data is freed.

**`Arc::clone(&arc)`** is CHEAP - it just increments a counter, doesn't copy data.

#### Step 3: Understanding Semaphore

```rust
let semaphore = Arc::new(Semaphore::new(config.concurrency as usize));
```

A semaphore is like a bouncer at a club - only lets N people in at once:

```
Concurrency = 3:

Time ────────────────────────────────────►

Task 1: ████████░░░░░░░░  (running)
Task 2: ░░██████████░░░░  (running)
Task 3: ░░░░████████████  (running)
Task 4: ░░░░░░░░░░░░████  (waiting... then runs)
Task 5: ░░░░░░░░░░░░░░██  (waiting... then runs)
```

```rust
let _permit = semaphore.acquire().await.unwrap();
// Now we have a permit - we're "inside the club"
// When _permit is dropped, the permit is released
```

**Why `_permit`?**
- The underscore prefix means "I know I'm not using this directly"
- But holding it is important! When dropped, permit is released.

#### Step 4: Spawning Tasks

```rust
for _ in 0..config.num_requests {
    // Clone Arc handles (cheap - just increments counter)
    let client = Arc::clone(&client);
    let semaphore = Arc::clone(&semaphore);
    let url = config.url.clone();  // Clone the actual string
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
```

**`tokio::spawn(async move { ... })`**
- Creates a new async task that runs concurrently
- `async move` - the closure takes ownership of captured variables
- Returns a `JoinHandle` we can await later

**Why clone everything?**
- Each task runs independently, potentially in parallel
- Each needs its own "copy" of the data
- `Arc::clone` is cheap (reference count)
- `String::clone` actually copies (but necessary)

#### Step 5: Collecting Results

```rust
let mut results = Vec::with_capacity(config.num_requests as usize);
for handle in handles {
    if let Ok(result) = handle.await {
        results.push(result);
    }
}
```

- `handle.await` - Wait for task to complete
- `if let Ok(result) = ...` - Only add successful task results
- Task might fail if it panics (unlikely but possible)

---

### Statistics Calculation

```rust
fn calculate_stats(results: &[RequestResult], total_duration: u128) -> LoadTestStats {
    let total_requests = results.len() as u64;
    let successful_requests = results.iter().filter(|r| r.success).count() as u64;
    let failed_requests = total_requests - successful_requests;

    // Get latencies from successful requests
    let mut latencies: Vec<u128> = results
        .iter()
        .filter(|r| r.success)
        .map(|r| r.duration)
        .collect();

    // Sort for percentile calculation
    latencies.sort_unstable();

    // Calculate stats...
}
```

#### Iterator Chain Explained:

```rust
let mut latencies: Vec<u128> = results
    .iter()              // Create iterator over results
    .filter(|r| r.success)  // Keep only successful ones
    .map(|r| r.duration)    // Extract just the duration
    .collect();             // Collect into Vec<u128>
```

Step by step:
```
results: [RequestResult, RequestResult, RequestResult, ...]
         ↓ .iter()
Iterator: &RequestResult, &RequestResult, &RequestResult, ...
         ↓ .filter(|r| r.success)
Iterator: &RequestResult, &RequestResult, ... (only successful)
         ↓ .map(|r| r.duration)
Iterator: 150, 200, 180, ... (just the durations)
         ↓ .collect()
Vec<u128>: [150, 200, 180, ...]
```

#### Why `sort_unstable`?

```rust
latencies.sort_unstable();
```

- "Unstable" means equal elements might be reordered
- Faster than stable sort
- We don't care about order of equal values
- Required for percentile calculation (data must be sorted)

---

### Percentile Calculation

```rust
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
        // Linear interpolation
        let weight = rank - lower as f64;
        let lower_val = sorted_data[lower] as f64;
        let upper_val = sorted_data[upper] as f64;
        (lower_val + weight * (upper_val - lower_val)) as u128
    }
}
```

#### What are Percentiles?

Percentiles tell you how your data is distributed:

| Percentile | Meaning |
|------------|---------|
| **p50 (median)** | 50% of requests were faster than this |
| **p95** | 95% of requests were faster than this |
| **p99** | 99% of requests were faster than this |

**Example:** If p50 = 100ms and p99 = 2000ms:
- Most requests are fast (~100ms)
- But 1% of users experience 2000ms (20x slower!)
- This reveals problems that averages hide.

#### The Math:

For 100 data points and p95:
1. `rank = (95/100) * 99 = 94.05`
2. `lower = 94`, `upper = 95`
3. `weight = 0.05`
4. Interpolate between `data[94]` and `data[95]`

**Linear Interpolation:**
```
value = lower_val + weight * (upper_val - lower_val)
```

This gives smooth values even when the percentile falls between data points.

---

## The Interactive Module (interactive.rs)

This module provides a guided TUI experience.

### Dialoguer Prompts

#### Text Input with Validation

```rust
let url: String = Input::with_theme(&theme)
    .with_prompt("Target URL")
    .with_initial_text("https://")
    .validate_with(|input: &String| -> Result<(), &str> {
        if input.starts_with("http://") || input.starts_with("https://") {
            Ok(())
        } else {
            Err("URL must start with http:// or https://")
        }
    })
    .interact_text()?;
```

| Method | Purpose |
|--------|---------|
| `.with_theme(&theme)` | Use colorful styling |
| `.with_prompt("...")` | Text shown to user |
| `.with_initial_text("...")` | Pre-filled text |
| `.validate_with(\|...\|)` | Validation closure |
| `.interact_text()` | Show prompt, get input |

**What user sees:**
```
? Target URL: https://|
                      ^ cursor here
```

#### Select (Dropdown)

```rust
let methods = vec!["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD"];
let method_index = Select::with_theme(&theme)
    .with_prompt("HTTP Method")
    .items(&methods)
    .default(0)
    .interact()?;
```

**What user sees:**
```
? HTTP Method:
> GET        ◄── highlighted, use ↑↓ to move
  POST
  PUT
  DELETE
  PATCH
  HEAD
```

Returns the **index** of selected item (0, 1, 2, etc.)

#### MultiSelect (Checkboxes)

```rust
let options = vec!["Option A", "Option B", "Option C"];
let selections = MultiSelect::with_theme(&theme)
    .with_prompt("Select options (Space to toggle, Enter to confirm)")
    .items(&options)
    .interact()?;
```

**What user sees:**
```
? Select options:
  [ ] Option A     ◄── Press Space to toggle
  [x] Option B     ◄── Selected!
  [ ] Option C
```

Returns `Vec<usize>` of selected indices.

#### Confirm (Yes/No)

```rust
let confirmed = Confirm::with_theme(&theme)
    .with_prompt("Start load test?")
    .default(true)
    .interact()?;
```

**What user sees:**
```
? Start load test? (Y/n): |
```

Returns `bool`.

### Input Validation

```rust
.validate_with(|input: &u64| -> Result<(), &str> {
    if *input > 0 {
        Ok(())
    } else {
        Err("Must be at least 1")
    }
})
```

- Runs on every keystroke
- `Ok(())` - Valid, allow submission
- `Err("message")` - Invalid, show error

**What user sees on invalid input:**
```
? Number of requests: 0
  ⚠ Must be at least 1
```

---

## The Main Module (main.rs)

Entry point and orchestration.

### CLI Argument Parsing

```rust
#[derive(Parser, Debug)]
#[command(author, version, about = "RustyLoad - A blazingly fast HTTP load testing tool")]
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

    /// Run in interactive mode
    #[clap(short, long)]
    interactive: bool,

    /// Skip confirmation
    #[clap(short = 'y', long)]
    yes: bool,
}
```

#### Attribute Explanations:

| Attribute | Purpose |
|-----------|---------|
| `#[derive(Parser)]` | Generate CLI parsing code |
| `#[command(...)]` | Program metadata |
| `/// Comment` | Becomes help text! |
| `#[clap(short, long)]` | Accept `-u` and `--url` |
| `#[clap(short = 'n')]` | Use `-n` instead of first letter |
| `Option<T>` | Optional argument |
| `bool` | Flag (present = true) |

#### Resulting CLI:

```
RustyLoad - A blazingly fast HTTP load testing tool

Usage: rustyload [OPTIONS]

Options:
  -u, --url <URL>                  Target URL to test
  -n, --requests <REQUESTS>        Number of requests to send
  -c, --concurrency <CONCURRENCY>  Number of concurrent requests
  -i, --interactive                Run in interactive mode
  -y, --yes                        Skip confirmation
  -h, --help                       Print help
  -V, --version                    Print version
```

### Mode Selection

```rust
// Determine mode
let use_interactive = args.interactive || args.url.is_none();

let config = if use_interactive {
    // Interactive mode
    interactive::run_interactive_mode(args.url)?
} else {
    // Quick mode
    let url = args.url.unwrap();
    let requests = args.requests.unwrap_or(100);
    let concurrency = args.concurrency.unwrap_or(10);

    client::LoadTestConfig::new(url, requests, concurrency)
};
```

**Decision Logic:**
```
Use interactive mode if:
  - User passed -i flag  OR
  - User didn't provide a URL

Otherwise: Quick mode with CLI args
```

**`.unwrap_or(100)`** - If `None`, use 100 as default.

---

## Key Rust Concepts Used

### Ownership and Borrowing

| Concept | Symbol | Meaning |
|---------|--------|---------|
| Ownership | `value` | You own it, you're responsible |
| Immutable borrow | `&value` | Temporary read access |
| Mutable borrow | `&mut value` | Temporary write access |

### Common Types

| Type | Meaning |
|------|---------|
| `String` | Owned, growable string |
| `&str` | Borrowed string slice |
| `Vec<T>` | Owned, growable array |
| `&[T]` | Borrowed slice |
| `Option<T>` | Maybe a value |
| `Result<T, E>` | Success or error |

### Error Handling

```rust
// The ? operator
let result = something_that_might_fail()?;
// If error, return early with that error
// If success, unwrap the value

// .context() adds message to errors
let file = File::open("x.txt").context("Failed to open config")?;

// match for custom handling
match result {
    Ok(value) => println!("Got: {}", value),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Async/Await

```rust
// Async function
async fn fetch_data() -> Result<String> {
    let response = client.get(url).send().await?;
    let text = response.text().await?;
    Ok(text)
}

// .await pauses until complete
// Other tasks can run while waiting
```

---

## Testing

### Running Tests

```bash
cargo test              # Run all tests
cargo test test_name    # Run specific test
cargo test -- --nocapture  # Show println! output
```

### Test Structure

```rust
#[cfg(test)]           // Only compile for tests
mod tests {
    use super::*;      // Import from parent module

    #[test]            // Mark as test function
    fn test_something() {
        // Arrange
        let input = 5;
        
        // Act
        let result = double(input);
        
        // Assert
        assert_eq!(result, 10);
    }
}
```

### Assertion Macros

| Macro | Purpose |
|-------|---------|
| `assert!(expr)` | Panic if false |
| `assert_eq!(a, b)` | Panic if a ≠ b |
| `assert_ne!(a, b)` | Panic if a = b |
| `assert!(matches!(val, pattern))` | Pattern matching |

---

## GitHub Actions CI/CD

We have two workflow files:
1. `ci.yml` - Runs on every push/PR for testing
2. `release.yml` - Runs when you create a version tag for releases

### CI Workflow (ci.yml)

#### Workflow Triggers

```yaml
on:
  push:
    branches: [ main, master ]
  pull_request:
    branches: [ main, master ]
```

Runs on:
- Push to main/master
- Pull requests to main/master

#### Matrix Strategy

```yaml
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
    rust: [stable, beta]
```

Creates **6 jobs** (3 OS × 2 Rust versions):
- ubuntu + stable
- ubuntu + beta
- macos + stable
- macos + beta
- windows + stable
- windows + beta

#### Caching

```yaml
- uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

Caches dependencies between runs for faster builds.

---

### Release Workflow (release.yml)

This workflow automatically builds and publishes binaries when you create a version tag.

#### Workflow Trigger

```yaml
on:
  push:
    tags:
      - 'v*'  # Triggers on tags like v0.1.0, v1.0.0, etc.
```

#### Build Matrix

```yaml
matrix:
  platform:
    # Linux
    - os: ubuntu-latest
      target: x86_64-unknown-linux-gnu
      bin: rustyload
      name: rustyload-linux-x86_64.tar.gz

    # macOS Intel
    - os: macos-latest
      target: x86_64-apple-darwin
      bin: rustyload
      name: rustyload-macos-x86_64.tar.gz

    # macOS Apple Silicon
    - os: macos-latest
      target: aarch64-apple-darwin
      bin: rustyload
      name: rustyload-macos-aarch64.tar.gz

    # Windows
    - os: windows-latest
      target: x86_64-pc-windows-msvc
      bin: rustyload.exe
      name: rustyload-windows-x86_64.zip
```

Builds binaries for:
- **Linux** (x86_64)
- **macOS Intel** (x86_64)
- **macOS Apple Silicon** (M1/M2/M3 - aarch64)
- **Windows** (x86_64)

#### Packaging

Unix systems get `.tar.gz`:
```yaml
- name: Package (Unix)
  if: runner.os != 'Windows'
  run: |
    cd target/${{ matrix.platform.target }}/release
    tar czvf ../../../${{ matrix.platform.name }} ${{ matrix.platform.bin }}
```

Windows gets `.zip`:
```yaml
- name: Package (Windows)
  if: runner.os == 'Windows'
  run: |
    cd target/${{ matrix.platform.target }}/release
    7z a ../../../${{ matrix.platform.name }} ${{ matrix.platform.bin }}
```

#### Creating a Release

The workflow uses `softprops/action-gh-release` to create a GitHub Release:

```yaml
- name: Create Release
  uses: softprops/action-gh-release@v1
  with:
    files: artifacts/**/*
    generate_release_notes: true
```

---

### How to Create a Release

#### Step 1: Update Version

Edit `Cargo.toml`:
```toml
[package]
version = "0.1.0"  # Change this
```

#### Step 2: Commit Changes

```bash
git add .
git commit -m "Prepare for v0.1.0 release"
git push origin main
```

#### Step 3: Create and Push Tag

```bash
# Create a tag
git tag v0.1.0

# Push the tag (this triggers the release workflow!)
git push origin v0.1.0
```

#### Step 4: Wait for Build

GitHub Actions will:
1. Build binaries for all 4 platforms
2. Package them (`.tar.gz` for Unix, `.zip` for Windows)
3. Create a GitHub Release
4. Attach all binaries to the release

#### Step 5: Users Download

Users can download from:
`https://github.com/yourusername/rustyload/releases`

| Platform | File |
|----------|------|
| Linux | `rustyload-linux-x86_64.tar.gz` |
| macOS Intel | `rustyload-macos-x86_64.tar.gz` |
| macOS M1/M2/M3 | `rustyload-macos-aarch64.tar.gz` |
| Windows | `rustyload-windows-x86_64.zip` |

---

### Installation Instructions for Users

#### Linux

```bash
curl -LO https://github.com/yourusername/rustyload/releases/latest/download/rustyload-linux-x86_64.tar.gz
tar xzf rustyload-linux-x86_64.tar.gz
sudo mv rustyload /usr/local/bin/
rustyload --version
```

#### macOS

```bash
# Intel Mac
curl -LO https://github.com/yourusername/rustyload/releases/latest/download/rustyload-macos-x86_64.tar.gz

# Apple Silicon (M1/M2/M3)
curl -LO https://github.com/yourusername/rustyload/releases/latest/download/rustyload-macos-aarch64.tar.gz

tar xzf rustyload-macos-*.tar.gz
sudo mv rustyload /usr/local/bin/
rustyload --version
```

#### Windows

1. Download `rustyload-windows-x86_64.zip` from Releases
2. Extract the `.zip` file
3. Move `rustyload.

---

## Conclusion

You now understand:

1. **How Rust manages memory** with ownership and borrowing
2. **How async works** with Tokio, spawn, and await
3. **How to build CLIs** with clap's derive macros
4. **How to make interactive TUIs** with dialoguer
5. **How to handle errors** with anyhow and Result
6. **How to test** with Rust's built-in test framework
7. **How to set up CI/CD** with GitHub Actions

This is production-quality Rust code that demonstrates:
- Systems programming skills
- Async/concurrent programming
- Clean architecture
- Testing best practices
- DevOps awareness

Good luck!