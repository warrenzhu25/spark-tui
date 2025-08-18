# Spark TUI - Terminal UI for Apache Spark Event Log Analysis

A terminal-based user interface for analyzing Apache Spark event logs, providing similar functionality to the Spark Web UI but in a terminal environment.

## Features

- **Jobs Overview**: View job status, duration, stages, and task progress
- **Stages Overview**: Analyze stage performance, task distribution, and RDD information  
- **Executors Overview**: Monitor executor resource usage, task distribution, and performance metrics
- **Environment Overview**: Browse Spark, system, and Hadoop configuration properties
- **Navigation**: Keyboard-driven interface with vim-like navigation

## Installation

### Prerequisites

- Rust toolchain (1.70+)

### Build from Source

```bash
git clone <repository-url>
cd spark-tui
cargo build --release
```

## Usage

### Run with Event Log

```bash
# Basic usage
cargo run -- --log-file /path/to/spark-eventlog

# Or with the built binary
./target/release/spark-tui --log-file /path/to/spark-eventlog
```

### Navigation

- **Tab/Shift+Tab**: Switch between tabs
- **1-4**: Jump directly to tabs (Jobs, Stages, Executors, Environment)
- **↑↓ or j/k**: Navigate within tables
- **q/Esc**: Quit application

### Event Log Location

Spark event logs are typically found in:
- `spark.eventLog.dir` (configured in Spark)
- Default: `/tmp/spark-events/` or `$SPARK_HOME/logs/`
- Application logs: `<eventlog-dir>/<app-id>`

## Screenshots

The application provides four main views:

1. **Jobs Tab**: Lists all Spark jobs with status, timing, and progress information
2. **Stages Tab**: Shows stage details including task counts and execution status  
3. **Executors Tab**: Displays executor resource usage and task distribution
4. **Environment Tab**: Shows Spark configuration properties and system information

## Supported Event Log Format

This tool parses Spark event logs in JSON format. It supports the standard Spark event log structure including:

- Application lifecycle events
- Job and stage execution events  
- Task completion and metrics
- Executor management events
- Environment configuration

## Sample Event Log

A sample event log is included (`sample-eventlog.json`) for testing purposes.

## Architecture

The application is built with:

- **ratatui**: Terminal UI framework
- **crossterm**: Cross-platform terminal handling  
- **serde**: JSON parsing and serialization
- **chrono**: Date/time handling
- **clap**: Command line argument parsing

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT OR Apache-2.0 license.