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
- **🎯 Interactive Mode** - Guided TUI for easy configuration (no need to memorize flags!)
- **🔧 HTTP Methods** - Support for GET, POST, PUT, DELETE, PATCH, and HEAD
- **📝 Custom Headers** - Add any custom headers including Authorization
- **📦 Request Body** - Send JSON or any payload with POST/PUT/PATCH requests
- **⏱️ Configurable Timeout** - Set request timeout in seconds
- **🎨 Beautiful TUI** - Colorful terminal output with progress bar
- **📈 Real-time Progress** - Live progress bar showing request completion
- **🛡️ Error Handling** - Graceful handling of failed requests with detailed reporting

---

## 📦 Installation

### Option 1: Download Pre-built Binary (Easiest)

Download the latest release for your platform from the [Releases page](https://github.com/yourusername/rustyload/releases).

#### Linux

```bash
# Download the latest release
curl -LO https://github.com/yourusername/rustyload/releases/latest/download/rustyload-linux-x86_64.tar.gz

# Extract the binary
tar xzf rustyload-linux-x86_64.tar.gz

# Move to a directory in your PATH
sudo mv rustyload /usr/local/bin/

# Verify installation
rustyload --version
```

#### macOS

```bash
# For Intel Macs
curl -LO https://github.com/yourusername/rustyload/releases/latest/download/rustyload-macos-x86_64.tar.gz
tar xzf rustyload-macos-x86_64.tar.gz

# For Apple Silicon (M1/M2/M3)
curl -LO https://github.com/yourusername/rustyload/releases/latest/download/rustyload-macos-aarch64.tar.gz
tar xzf rustyload-macos-aarch64.tar.gz

# Move to PATH
sudo mv rustyload /usr/local/bin/

# Verify installation
rustyload --version
```

#### Windows

1. Download `rustyload-windows-x86_64.zip` from the [Releases page](https://github.com/yourusername/rustyload/releases)
2. Extract the `.zip` file
3. Move `rustyload.exe` to a directory in your PATH (e.g., `C:\Windows\System32\` or create a custom directory)
4. Or run directly: `.\rustyload.exe --version`

### Option 2: Install via Cargo

If you have Rust installed:

```bash
# Install directly from GitHub
cargo install --git https://github.com/yourusername/rustyload

# Or clone and install locally
git clone https://github.com/yourusername/rustyload
cd rustyload
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/rustyload`.

### Option 3: Build from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/rustyload
cd rustyload

# Build in release mode (optimized)
cargo build --release

# The binary will be at ./target/release/rustyload
./target/release/rustyload --version

# Optionally, copy to your PATH
sudo cp target/release/rustyload /usr/local/bin/
```

---

## 🚀 Usage

RustyLoad offers two modes: **Interactive Mode** (beginner-friendly) and **Quick Mode** (for power users).

### Interactive Mode (Recommended for Beginners)

Simply run without arguments, or use the `-i` flag:

```bash
# Launches interactive configuration wizard
rustyload

# Or explicitly request interactive mode
rustyload -i
```

The interactive mode will guide you through:
1. ✅ Target URL
2. ✅ HTTP Method (GET, POST, PUT, DELETE, etc.)
3. ✅ Number of requests
4. ✅ Concurrency level
5. ✅ Timeout settings
6. ✅ Custom headers (optional)
7. ✅ Request body (optional)

```
🚀 Interactive Mode - Let's configure your load test!
──────────────────────────────────────────────────

? Target URL: https://api.example.com/health
? HTTP Method: GET
? Number of requests: 100
? Concurrent requests: 10
? Timeout (seconds): 30
? Additional options: Skip (use defaults)

──────────────────────────────────────────────────
✅ Configuration complete!
```

### Quick Mode (For Power Users)

If you know what you want, use CLI arguments:

```bash
# Basic usage with defaults (100 requests, 10 concurrent)
rustyload -u https://httpbin.org/get

# Custom requests and concurrency
rustyload -u https://api.example.com/health -n 500 -c 50

# Skip confirmation prompt with -y
rustyload -u https://httpbin.org/get -n 100 -c 10 -y
```

### Command Line Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--url` | `-u` | Target URL to test | - |
| `--requests` | `-n` | Total number of requests to send | 100 |
| `--concurrency` | `-c` | Number of concurrent requests | 10 |
| `--interactive` | `-i` | Run in interactive mode | auto |
| `--yes` | `-y` | Skip confirmation prompt | false |
| `--help` | `-h` | Show help message | - |
| `--version` | `-V` | Show version | - |

> **Note:** If you don't provide a URL, RustyLoad automatically enters interactive mode!

### Example Output

```
┌─────────────────────────────────────────────────┐
│ 📋 Configuration Summary                         │
├─────────────────────────────────────────────────┤
│ URL:               https://httpbin.org/post     │
│ Method:            POST                         │
│ Requests:          100                          │
│ Concurrency:       10                           │
│ Timeout:           30s                          │
├─────────────────────────────────────────────────┤
│ Headers:                                        │
│   Content-Type: application/json                │
│   Authorization: Bearer t...                    │
├─────────────────────────────────────────────────┤
│ Body:              {"name":"test"}              │
└─────────────────────────────────────────────────┘

? Start load test? Yes

🚀 Starting load test...

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

1. **Configure** - Interactive prompts or CLI args
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
│       ├── ci.yml              # GitHub Actions CI/CD
│       └── release.yml         # Automated release builds
├── docs/
│   └── TECHNICAL_GUIDE.md      # Detailed technical documentation
├── src/
│   ├── main.rs                 # CLI parsing, orchestration
│   ├── client.rs               # HTTP client, load testing, statistics
│   └── interactive.rs          # Interactive TUI prompts
├── Cargo.toml                  # Dependencies and metadata
├── README.md                   # This file
└── LICENSE                     # MIT License
```

### Module Breakdown

#### `main.rs`
- **CLI Parsing**: Uses `clap` for argument parsing
- **Mode Selection**: Chooses between interactive and quick mode
- **Results Display**: Shows statistics in a beautiful table

#### `client.rs`
- **HTTP Client**: Built on `reqwest` with custom configuration
- **Concurrency**: Tokio-based async with semaphore limiting
- **Statistics**: Percentile calculation with linear interpolation
- **Progress**: Real-time progress bar using `indicatif`
- **Configuration**: Builder pattern for flexible test setup

#### `interactive.rs`
- **Guided Setup**: Step-by-step configuration wizard
- **Input Validation**: Validates URLs, numbers, headers
- **Smart Defaults**: Sensible defaults for quick setup
- **User-Friendly**: Clear prompts with helpful descriptions

### Key Dependencies

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime for concurrent execution |
| `reqwest` | HTTP client for making requests |
| `clap` | Command-line argument parsing |
| `dialoguer` | Interactive terminal prompts |
| `indicatif` | Progress bar and spinners |
| `colored` | Terminal colors and styling |
| `anyhow` | Ergonomic error handling |

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
- **Unit tests** for HTTP method parsing
- **Unit tests** for configuration builder

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
- [ ] **Saved Profiles**: Save and load test configurations

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
- [dialoguer](https://github.com/console-rs/dialoguer) - Interactive prompts
- [indicatif](https://github.com/console-rs/indicatif) - Progress bars
- Inspired by tools like [wrk](https://github.com/wg/wrk), [hey](https://github.com/rakyll/hey), and [bombardier](https://github.com/codesenberg/bombardier)

---

<div align="center">

**Built with ❤️ and 🦀**

[⬆ Back to Top](#-rustyload)

</div>