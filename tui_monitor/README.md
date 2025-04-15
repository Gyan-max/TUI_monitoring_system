# TUI System Monitor

A terminal-based system monitoring tool built with Rust, Ratatui, and Tokio. This application displays system metrics (CPU, memory) and enables Tokio runtime instrumentation for async application monitoring.

![CPU Monitor Screenshot](assets/Screenshot%20(7).png)
![Memory Monitor Screenshot](assets/Screenshot%20(8).png)

## Features

- Real-time system resource monitoring (CPU, memory)
- Tokio runtime instrumentation support
- Clean terminal UI using Ratatui

## Prerequisites

- Rust and Cargo (latest stable)

## Installation

Clone the repository and build the project:

```bash
git clone https://github.com/Gyan-max/TUI_monitoring_system.git
cd tui_monitor
cargo build --release
```

## Usage

### Basic Monitoring

To run the application with basic system monitoring:

```bash
cargo run --release
```

### Tokio Runtime Monitoring

For Tokio runtime monitoring, you need to run the application with console instrumentation and then connect with tokio-console.

1. Install tokio-console (in a separate terminal):
   ```bash
   cargo install tokio-console
   ```

2. Run your application with the necessary environment variables:
   ```bash
   # On Windows
   $env:RUSTFLAGS="--cfg tokio_unstable"
   cargo run --release
   
   # On Linux/macOS
   RUSTFLAGS="--cfg tokio_unstable" cargo run --release
   ```

3. In another terminal, run tokio-console to connect to your application:
   ```bash
   tokio-console
   ```

## Controls

- Press `q` to quit the application

## Development

This project uses:
- `ratatui` for the terminal UI
- `crossterm` for terminal manipulation
- `sysinfo` for system metrics collection
- `tokio` and `console-subscriber` for async runtime instrumentation

## License

MIT 
