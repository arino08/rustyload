# 🦀 RustyLoad

<div align="center">

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Tokio](https://img.shields.io/badge/Tokio-async-blue?style=for-the-badge)
![License](https://img.shields.io/badge/License-MIT-green?style=for-the-badge)

**A blazingly fast, concurrent HTTP load testing tool built in Rust**

[Features](#-features) •
[Installation](#-installation) •
[Usage](#-usage) •
[How It Works](#-how-it-works) •
[Architecture](#-architecture) •
[Contributing](#-contributing)

</div>

---

## 📖 Overview

RustyLoad is a command-line HTTP load testing tool designed to stress-test web servers and APIs. Built with Rust's async runtime (Tokio), it can send thousands of concurrent requests while efficiently managing system resources.

Whether you're testing your local development server or benchmarking a production API, RustyLoad provides detailed latency statistics including percentiles (p50, p95, p99) to help you understand your server's performance characteristics.

```
  ____           _         _                    _
 |  _ \ _   _ __| |_ _   _| |    ___   __ _  __| |
 | |_) | | | / _` __| | | | |   / _ \ / _` |/ _` |
 |  _ <| |_| \__ \ |_| |_| | |__| (_) | (_| | (_| |
 |_| \_\\__,_|___/\__|\__, |_____\___/ \__,_|\__,_|
                      |___/

  ⚡ Blazingly Fast HTTP Load Testing Tool ⚡
```

---

## ✨ Features

- **🚀 High Performance** - Built with Rust and Tokio for maximum throughput
- **⚡ Concurrent Requests** - Control concurrency level with semaphore-based limiting
- **📊 Detailed Statistics** - Min, max, average latency plus p50, p95, p99 percentiles
- **🔧 HTTP Methods** - Support for GET, POST, PUT, DELETE, PATCH, and HEAD
- **📝 Custom Headers** - Add any custom headers to your requests
- **📦 Request Body** - Send JSON or any payload with POST/PUT/PATCH requests
- **⏱️ Configurable Timeout** - Set request timeout in seconds
- **🎨 Beautiful TUI** - Colorful terminal output with progress bar
- **📈 Real-time Progress** - Live progress bar showing request completion
- **🛡️ Error Handling** - Graceful handling of failed requests with detailed reporting

---

## 📦 Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rustyload.git
cd rustyload

# Build in release mode (optimized)
cargo build --release

# The binary will be at ./target/release/rustyload
```

### Install via Cargo

```bash
cargo install --path .
```

---

## 🚀 Usage

### Basic Usage

```bash
# Send 100 requests with 10 concurrent connections (defaults)
rustyload --url https://httpbin.org/get

# Short form
rustyload -u https://httpbin.org/get
```

### Custom Configuration

```bash
# Send 500 requests with 50 concurrent connections
rustyload --url https://api.example.com/health --requests 500 --concurrency 50

# Short form
rustyload -u https://api.example.com/health -n 500 -c 50
```

### POST Request with JSON Body

```bash
rustyload -u https://httpbin.org/post \
  -m POST \
  -H "Content-Type: application/json" \
  -b '{"name": "test", "value": 123}' \
  -n 100 -c 10
```

### With Custom Headers and Authentication

```bash
rustyload -u https://api.example.com/data \
  -m GET \
  -H "Authorization: Bearer your-token-here" \
  -H "X-Custom-Header: custom-value" \
  -n 200 -c 20
```

### PUT Request with Timeout

```bash
rustyload -u https://api.example.com/resource/1 \
  -m PUT \
  -H "Content-Type: application/json" \
  -b '{"updated": true}' \
  -t 60 \
  -n 50 -c 5
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--url` | `-u` | Target URL to test (required) | - |
| `--requests` | `-n` | Total number of requests to send | 100 |
| `--concurrency` | `-c` | Number of concurrent requests | 10 |
| `--method` | `-m` | HTTP method (GET, POST, PUT, DELETE, PATCH, HEAD) | GET |
| `--header` | `-H` | Custom header (can be used multiple times) | - |
| `--body` | `-b` | Request body for POST/PUT/PATCH | - |
| `--timeout` | `-t` | Request timeout in seconds | 30 |
| `--help` | `-h` | Show help message | - |
| `--version` | `-V` | Show version | - |

### Example Output

```
┌─────────────────────────────────────────────────┐
│ Configuration                                   │
├─────────────────────────────────────────────────┤
│ Target:         https://httpbin.org/post        │
│ Method:         POST                            │
│ Requests:       100                             │
│ Concurrency:    10                              │
│ Timeout:        30 seconds                      │
├─────────────────────────────────────────────────┤
│ Custom Headers                                  │
│ Content-Type: application/json                  │
│ Authorization: Bearer token123                  │
├─────────────────────────────────────────────────┤
│ Body:           {"name":"test"}                 │
└─────────────────────────────────────────────────┘

Starting load test...

  [00:00:12] [████████████████████████████████████████] 100/100 (100%) Complete!

┌─────────────────────────────────────────────────┐
│ 📊 Results                                       │
├─────────────────────────────────────────────────┤
│ Total Requests:      100                        │
│ Successful:          100 (100.0%)               │
│ Failed:              0                          │
├─────────────────────────────────────────────────┤
│ ⏱️  Latency (ms)                                │
├─────────────────────────────────────────────────┤
│ Min:                 145 ms                     │
│ Max:                 892 ms                     │
│ Average:             234.56 ms                  │
├─────────────────────────────────────────────────┤
│ 📈 Percentiles                                   │
├─────────────────────────────────────────────────┤
│ p50 (median):        210 ms                     │
│ p95:                 445 ms                     │
│ p99:                 823 ms                     │
├─────────────────────────────────────────────────┤
│ 🚀 Throughput                                    │
├─────────────────────────────────────────────────┤
│ Requests/sec:        8.23                       │
│ Total time:          12156 ms                   │
└─────────────────────────────────────────────────┘

✅ Load test completed successfully!
```

---

## 🔍 How It Works

### Understanding Percentiles

Percentiles help you understand the distribution of response times:

| Percentile | Meaning |
|------------|---------|
| **p50 (Median)** | 50% of requests were faster than this value |
| **p95** | 95% of requests were faster - shows "almost worst case" |
| **p99** | 99% of requests were faster - catches outliers |

**Why percentiles matter:** If your average latency is 100ms but p99 is 2000ms, it means 1% of your users experience 20x slower response times!

### Concurrency Control

RustyLoad uses a **semaphore** to control concurrency:

```
Concurrency = 3:

Request 1: ████████░░░░░░░░  (running)
Request 2: ░░██████████░░░░  (running)
Request 3: ░░░░████████████  (running)
Request 4: ░░░░░░░░░░░░████  (waiting, then runs when slot opens)
```

This prevents overwhelming both your system and the target server.

### Request Flow

1. **Parse CLI arguments** - Validate URL, method, headers, body
2. **Build HTTP client** - Configure timeout, user agent
3. **Create semaphore** - Limit concurrent requests
4. **Spawn async tasks** - One task per request
5. **Collect results** - Gather timing and status from each request
6. **Calculate statistics** - Compute percentiles, averages, throughput
7. **Display results** - Pretty-print in terminal

---

## 🏗️ Architecture

```
rustyload/
├── .github/
│   └── workflows/
│       └── ci.yml          # GitHub Actions CI/CD
├── src/
│   ├── main.rs             # CLI parsing, TUI, orchestration
│   └── client.rs           # HTTP client, load testing, statistics
├── Cargo.toml              # Dependencies and metadata
├── README.md               # Documentation
└── LICENSE                 # MIT License
```

### Module Breakdown

#### `main.rs`
- **CLI Parsing**: Uses `clap` for argument parsing with derive macros
- **TUI Rendering**: Colorful output using `colored` crate
- **Orchestration**: Coordinates the flow from input to output

#### `client.rs`
- **HTTP Client**: Built on `reqwest` with custom configuration
- **Concurrency**: Tokio-based async with semaphore limiting
- **Statistics**: Percentile calculation with linear interpolation
- **Progress**: Real-time progress bar using `indicatif`
- **Configuration**: Builder pattern for flexible test setup

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime for concurrent execution |
| `reqwest` | HTTP client for making requests |
| `clap` | Command-line argument parsing |
| `indicatif` | Progress bar and spinners |
| `colored` | Terminal colors and styling |
| `anyhow` | Ergonomic error handling |

---

## 🧪 Testing

RustyLoad includes a comprehensive test suite:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_percentile
```

### Test Coverage

- **Unit tests** for percentile calculations
- **Unit tests** for statistics aggregation
- **Unit tests** for header parsing
- **Unit tests** for HTTP method parsing
- **Unit tests** for configuration builder

---

## 📊 Performance Characteristics

RustyLoad is designed to be efficient:

- **Memory**: Uses `Arc` for shared state, avoiding unnecessary clones
- **CPU**: Async I/O means threads aren't blocked waiting for responses
- **Network**: Reuses HTTP client connections where possible

### Benchmarks

Tested on a local server (your results may vary):

| Requests | Concurrency | Time | Req/sec |
|----------|-------------|------|---------|
| 1,000 | 10 | 2.3s | 434 |
| 1,000 | 50 | 0.8s | 1,250 |
| 10,000 | 100 | 5.2s | 1,923 |

---

## 🛠️ Development

### Running Locally

```bash
# Debug build (faster compile, slower runtime)
cargo build

# Run directly
cargo run -- -u https://httpbin.org/get -n 10

# Release build (slower compile, faster runtime)
cargo build --release
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for issues without building
cargo check
```

---

## 🗺️ Roadmap

Future enhancements planned:

- [ ] **Output Formats**: JSON, CSV export for CI/CD integration
- [ ] **HTML Reports**: Generate visual reports
- [ ] **Request from File**: Load URLs/requests from a file
- [ ] **Distributed Testing**: Run from multiple machines
- [ ] **Latency Histogram**: Visual ASCII distribution of response times
- [ ] **Rate Limiting**: Requests per second limiting
- [ ] **Duration Mode**: Run for X seconds instead of X requests
- [ ] **Cookies**: Cookie jar support for session testing
- [ ] **HTTP/2**: HTTP/2 protocol support
- [ ] **mTLS**: Mutual TLS authentication

---

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Development Guidelines

- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Add tests for new functionality
- Update README for user-facing changes

### Areas for Contribution

- 🐛 Bug fixes
- 📝 Documentation improvements
- ✨ New features from the roadmap
- 🧪 Test coverage
- 🎨 TUI improvements

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- [Tokio](https://tokio.rs/) - Async runtime for Rust
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [clap](https://github.com/clap-rs/clap) - Command line argument parser
- [indicatif](https://github.com/console-rs/indicatif) - Progress bars
- Inspired by tools like [wrk](https://github.com/wg/wrk), [hey](https://github.com/rakyll/hey), and [bombardier](https://github.com/codesenberg/bombardier)

---

<div align="center">

**Built with ❤️ and 🦀**

[⬆ Back to Top](#-rustyload)

</div>