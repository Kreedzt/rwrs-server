# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

RWRS Server is a Rust backend server for the [rwrs-another-page](https://github.com/Kreedzt/rwrs-another-page) frontend. It acts as a proxy for the Running with Rifles game server list API and serves static files.

## Architecture

This is a simple Rust web application using the Salvo framework:

- **Core server**: Single `main.rs` file with all application logic
- **Framework**: Salvo web framework with logging, request ID, proxy, and static file serving features
- **Proxy functionality**: Routes `/api/server_list` requests to `http://rwr.runningwithrifles.com/rwr_server_list/get_server_list.php`
- **Static serving**: Serves files from a `static/` directory (defaults to `index.html` for unknown paths)
- **Configuration**: Uses environment variables `HOST` (default: `127.0.0.1`) and `PORT` (default: `5800`)

## Development Commands

### Local Development
```bash
# Run development server
cargo run

# Custom host/port
HOST=0.0.0.0 PORT=8080 cargo run
```

### Build
```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Docker
```bash
# Build Docker image
docker build -t rwrs-server .

# Run container
docker run -d -p 80:80 --name rwrs-server -e HOST=0.0.0.0 -e PORT=80 rwrs-server
```

## Project Structure

- `src/main.rs` - Main application file with all server logic
- `static/` - Directory for static files (created during deployment)
- `Cargo.toml` - Rust project configuration
- `Dockerfile` - Multi-stage build for production
- `README.md` - Detailed deployment and development instructions

## Key Dependencies

- `salvo` (0.76.2) - Web framework with proxy and static serving features
- `tokio` (1.43.0) - Async runtime
- `tracing` (0.1.41) - Logging and instrumentation

## Deployment Notes

The server expects a `static/` directory containing the frontend build files from rwrs-another-page. The Docker image uses a scratch base layer for minimal container size.