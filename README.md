# Gohangout-rs

A Rust implementation of [GoHangout](https://github.com/childe/gohangout) - a Logstash alternative for ETL processing.

## Features

- **Multiple Inputs**: Kafka, STDIN, TCP, UDP
- **Rich Filters**: Field operations, Grok parsing, Regex replacement, etc.
- **Multiple Outputs**: Elasticsearch, ClickHouse, Kafka, InfluxDB, STDOUT
- **Hot Reload**: Configuration file monitoring and reloading
- **Monitoring**: Prometheus metrics integration
- **High Performance**: Concurrent processing with memory safety

## Project Status

🚧 **Under Development** - This is a work in progress to port GoHangout from Go to Rust.

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

## License

MIT