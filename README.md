# grainx

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-30%2F30%20passing-brightgreen.svg)](#testing)

A modern, cross-platform, terminal-native system monitoring tool written in Rust.

## Features

- **High-Resolution Visualization**: Unicode Braille pattern graphics (8x higher resolution)
- **Real-time Monitoring**: CPU, memory, disk, and network usage
- **Intelligent Analytics**: Anomaly detection, correlation analysis, predictive forecasting
- **Interactive Interface**: Process management with keyboard controls
- **Cross-Platform**: Linux, macOS 10.12+, Windows 10+
- **Performance Optimized**: Adaptive refresh rates and memory efficient

## Quick Start

```bash
# Clone the repository
git clone https://github.com/rustfuture/grainx.git
cd grainx

# Run the monitor
cargo run
```

## Controls

| Key | Action |
|-----|--------|
| `q` / `ESC` | Exit |
| `↑` / `↓` | Navigate processes |
| `k` | Kill selected process |
| `h` / `?` | Help menu |
| `p` | Pause/Resume |
| `a` | Toggle adaptive refresh |

## Configuration

Edit `dashboard_config.json` to customize:

```json
{
  "refresh_interval_ms": 500,
  "cpu_warning_threshold": 80.0,
  "memory_warning_threshold": 85.0,
  "max_processes": 10
}
```

## Testing

```bash
cargo test    # Run tests
cargo bench   # Run benchmarks
```

## License

MIT License - see [LICENSE](LICENSE) file.

---

**Made with ❤️ in Rust**
