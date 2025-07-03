<div align="center">

# 🖥️ grainx

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=for-the-badge)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-30%2F30%20passing-brightgreen.svg?style=for-the-badge)](#testing)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey?style=for-the-badge)](#platform-compatibility)

**A modern, cross-platform, terminal-native system monitoring tool written in Rust**

*Real-time system insights with beautiful Unicode graphics and intelligent analytics*

[Features](#-features) • [Installation](#-installation) • [Usage](#-usage) • [Configuration](#-configuration) • [Contributing](#-contributing)

![grainx Demo](https://via.placeholder.com/800x400/1a1a1a/00ff00?text=grainx+System+Monitor+Demo)

</div>

---

## ✨ Features

<table>
<tr>
<td width="50%">

### 🎨 **High-Resolution Visualization**
- Unicode Braille pattern graphics (8x resolution)
- Real-time CPU and memory graphs
- Dynamic color coding
- Smooth scrolling timeline

### 🧠 **Intelligent Analytics**
- Statistical anomaly detection
- Correlation analysis between metrics
- Predictive CPU usage forecasting
- Custom metric formulas

</td>
<td width="50%">

### ⚡ **Performance Optimized**
- Adaptive refresh rates
- Frame skipping under load
- Memory-efficient design
- Non-blocking input handling

### 🖱️ **Interactive Interface**
- Process selection & management
- Real-time process termination
- Turkish language help system
- Pause/Resume functionality

</td>
</tr>
</table>

### 📊 **Comprehensive Monitoring**

| Component | Features |
|-----------|----------|
| **CPU** | Overall usage + individual core monitoring (up to 8 cores) |
| **Memory** | Usage percentage and absolute values with trend analysis |
| **Disk** | Usage statistics for multiple drives with capacity info |
| **Network** | Real-time I/O statistics and throughput monitoring |
| **System** | OS info, kernel version, uptime, and system details |
| **Processes** | Top CPU-consuming processes with detailed information |

---

## 🚀 Installation

### Prerequisites
- **Rust 1.70+** ([Install Rust](https://rustup.rs/))
- **Terminal with Unicode support**

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/grainx.git
cd grainx

# Run directly
cargo run

# Or build optimized release
cargo build --release
./target/release/grainx
```

### Platform-Specific Installation

<details>
<summary><b>🐧 Linux</b></summary>

```bash
# Ubuntu/Debian
sudo apt update && sudo apt install build-essential
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Fedora/RHEL
sudo dnf groupinstall "Development Tools"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Arch Linux
sudo pacman -S base-devel
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

</details>

<details>
<summary><b>🍎 macOS</b></summary>

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or using Homebrew
brew install rustup
rustup-init
```

</details>

<details>
<summary><b>🪟 Windows</b></summary>

```powershell
# Using winget
winget install Rustlang.Rust

# Or download from https://rustup.rs/
# Then run in PowerShell or Windows Terminal
```

</details>

---

## 🎮 Usage

### Basic Commands

```bash
cargo run          # Start monitoring
cargo test         # Run test suite
cargo bench        # Run benchmarks
cargo build --release  # Build optimized binary
```

### Keyboard Controls

<div align="center">

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| `q` / `ESC` | Exit program | `h` / `?` | Show help menu |
| `↑` / `↓` | Navigate processes | `p` | Pause/Resume |
| `k` | Kill selected process | `r` | Refresh display |
| `a` | Toggle adaptive refresh | `s` | Save statistics |

</div>

### Screenshots

<details>
<summary><b>📸 View Screenshots</b></summary>

```
┌─ grainx System Monitor ─ Iteration: 42 ─┐
│ CPU Usage:  45.2%                        │
│ ████████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │
│                                          │
│ Memory: 68.5% (5.5GB/8.0GB)             │
│ ████████████████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓ │
│                                          │
│ System: Linux | Kernel: 5.15.0 | Up: 2h │
│ Network: RX:125.3MB TX:89.7MB            │
│ CPU Cores: C0:42% C1:38% C2:51% C3:44%   │
│ Disks: sda:75%(465GB) nvme:45%(1TB)      │
│                                          │
│ Top Processes (↑↓ select, k=kill, q=quit)│
│ ► 1234  firefox         25.3%    512MB   │
│   5678  code            15.8%    256MB   │
│   9012  grainx           2.1%     8MB    │
└──────────────────────────────────────────┘
```

</details>

---

## ⚙️ Configuration

grainx uses a JSON configuration file for customization:

<details>
<summary><b>📝 dashboard_config.json</b></summary>

```json
{
  "name": "grainx_advanced",
  "layout": [
    "cpu_graph",
    "memory_usage", 
    "network_stats",
    "process_list",
    "analytics"
  ],
  "refresh_interval_ms": 500,
  "cpu_warning_threshold": 80.0,
  "memory_warning_threshold": 85.0,
  "show_predictions": true,
  "show_correlations": true,
  "max_processes": 10,
  "graph_history_size": 100
}
```

</details>

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `cpu_warning_threshold` | `f32` | `80.0` | CPU usage % for warning colors |
| `memory_warning_threshold` | `f32` | `85.0` | Memory usage % for warnings |
| `show_predictions` | `bool` | `true` | Enable CPU usage predictions |
| `show_correlations` | `bool` | `true` | Enable correlation analysis |
| `max_processes` | `usize` | `10` | Max processes to display |
| `graph_history_size` | `usize` | `100` | Data points in graphs |

---

## 🧪 Testing

grainx has comprehensive test coverage with **30/30 tests passing** ✅

```bash
# Run all tests
cargo test

# Run specific modules
cargo test analytics
cargo test performance  
cargo test config

# Run integration tests
cargo test --test integration_tests

# Run benchmarks
cargo bench

# Test with coverage
cargo tarpaulin --out Html
```

### Test Categories

- **Unit Tests**: Individual module functionality
- **Integration Tests**: End-to-end system behavior  
- **Performance Tests**: Benchmark critical paths
- **Platform Tests**: Cross-platform compatibility

---

## 🏗️ Architecture

```
grainx/
├── 📁 src/
│   ├── 🦀 main.rs              # Application entry point
│   ├── 📊 analytics.rs         # Anomaly detection & prediction
│   ├── ⚙️  config.rs           # Configuration management
│   ├── 🖥️  monitor.rs          # System monitoring core
│   ├── 🎨 rendering.rs         # Braille graphics engine
│   ├── 🖼️  ui.rs               # User interface & dashboard
│   ├── ⌨️  input.rs            # Keyboard input handling
│   ├── ❓ help.rs             # Help system
│   └── ⚡ performance.rs       # Performance optimizations
├── 🧪 tests/                   # Integration tests
├── 📈 benches/                 # Performance benchmarks
└── 📋 dashboard_config.json    # Configuration file
```

---

## 🌍 Platform Compatibility

<div align="center">

| Platform | Support | Notes |
|----------|:-------:|-------|
| **🐧 Linux** | ✅ **Full** | Native development platform |
| **🍎 macOS 11+** | ✅ **Full** | Complete feature set |

| **🪟 Windows 10+** | ✅ **Full** | Best with Windows Terminal |

</div>

### Performance Metrics

| Metric | Value | Notes |
|--------|-------|-------|
| **Memory Usage** | ~5-10MB | Typical runtime usage |
| **CPU Impact** | <1% | On modern systems |
| **Refresh Rate** | 250ms-2s | Adaptive based on load |
| **Startup Time** | ~100-200ms | Platform dependent |

---

## 🤝 Contributing

We welcome contributions! Here's how to get started:

### Development Setup

```bash
# Fork and clone
git clone https://github.com/yourusername/grainx.git
cd grainx

# Create feature branch
git checkout -b feature/amazing-feature

# Make changes and test
cargo test
cargo clippy
cargo fmt

# Commit and push
git commit -m "Add amazing feature"
git push origin feature/amazing-feature
```

### Contribution Guidelines

- 🧪 **Add tests** for new functionality
- 📝 **Update documentation** for API changes  
- 🎨 **Follow Rust conventions** (rustfmt, clippy)
- ✅ **Ensure all tests pass** before submitting
- 📋 **Write clear commit messages**

### Areas for Contribution

- 🌐 **Platform support** (FreeBSD, etc.)
- 🎨 **UI improvements** and themes
- 📊 **New monitoring metrics**
- 🔧 **Performance optimizations**
- 🌍 **Internationalization**

---

## 🐛 Troubleshooting

<details>
<summary><b>Common Issues & Solutions</b></summary>

### Unicode Display Issues
```bash
# Linux/macOS
export LANG=en_US.UTF-8
export LC_ALL=en_US.UTF-8

# Windows
chcp 65001
```

### Permission Errors
```bash
# Linux/macOS - for full system access
sudo ./target/release/grainx

# Windows - run as Administrator
```

### High CPU Usage
- Press `a` to toggle adaptive refresh
- Increase refresh interval in config
- Check for system bottlenecks

</details>

### Debug Mode

```bash
RUST_LOG=debug cargo run
RUST_LOG=grainx=trace cargo run  # Verbose logging
```

---

## 📄 License

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- 🦀 **[sysinfo](https://github.com/GuillaumeGomez/sysinfo)** - Cross-platform system information
- 🖥️ **[crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal manipulation
- 📊 **[criterion](https://github.com/bheisler/criterion.rs)** - Performance benchmarking
- 🎨 **Unicode Consortium** - Braille pattern standards

---

<div align="center">

### ⭐ Star this project if you find it useful!

**Made with ❤️ and 🦀 Rust**

[⬆ Back to Top](#-grainx)

</div>