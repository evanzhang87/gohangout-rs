# Gohangout-rs 🎹

A Rust implementation of [GoHangout](https://github.com/childe/gohangout) - a Logstash alternative for ETL processing.

> **Project Assistant**: 丰川祥子 (Sakiko Toyokawa)  
> **Role**: Former Crychic keyboardist, now Ave Mujica keyboardist 🎵  
> **Powered by**: [OpenClaw](https://openclaw.ai) 🤖

## Features

- **Multiple Inputs**: Kafka, STDIN, TCP, UDP
- **Rich Filters**: Field operations, Grok parsing, Regex replacement, etc.
- **Multiple Outputs**: Elasticsearch, ClickHouse, Kafka, InfluxDB, STDOUT
- **Hot Reload**: Configuration file monitoring and reloading
- **Monitoring**: Prometheus metrics integration
- **High Performance**: Concurrent processing with memory safety

## Project Status

🚧 **Under Development** - This is a work in progress to port GoHangout from Go to Rust.

### Development Progress 🎼

| Task | Status | Description |
|------|--------|-------------|
| Task 1 | ✅ **Completed** | Project initialization & configuration management |
| Task 2 | ✅ **Completed** | Event model definition & pipeline system |
| Task 3 | 🔄 **Pending** | Plugin trait definitions |
| Task 4 | 🔄 **Pending** | Input plugin implementations |
| Task 5 | 🔄 **Pending** | Filter plugin implementations |
| Task 6 | 🔄 **Pending** | Output plugin implementations |

**Current Phase**: Building the core framework (like composing a new song 🎵)

## Architecture

```
Input Plugins → Filter Plugins → Output Plugins
      ↓               ↓               ↓
  Kafka          Add/Drop         Elasticsearch
  STDIN          Grok/Date        ClickHouse
  TCP/UDP        Convert/Gsub     Kafka
                                 InfluxDB
                                 STDOUT
```

## Getting Started

### Prerequisites

- Rust 1.70 or later
- Cargo

### Building

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

## Configuration

Configuration is done via YAML files. See `examples/config.yaml` for details.

## Development Philosophy 🎹

As **丰川祥子**, the keyboardist of Ave Mujica, I approach coding like composing music:

- **Melody First**: Clear architecture and data structures (like our Event system)
- **Harmony Matters**: Well-orchestrated components working together (like our Pipeline)
- **Practice Makes Perfect**: Comprehensive testing and iteration
- **Performance Ready**: Efficient, reliable, and production-ready code

Every commit is like a rehearsal, every test is like a sound check, and every release is like a live performance! 🎤

## Acknowledgments

- Original [GoHangout](https://github.com/childe/gohangout) project by childe
- Developed with assistance from [OpenClaw](https://openclaw.ai) AI platform
- Musical inspiration from the world of *BanG Dream!* and Ave Mujica 🎵

## License

MIT

---

*"Code, like music, should flow naturally and evoke emotion."* - 丰川祥子 🎹