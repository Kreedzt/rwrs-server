# RWRS Server

Backend server for [rwrs-another-page](https://github.com/Kreedzt/rwrs-another-page).

## Introduction

This project serves as the backend for [rwrs-another-page](https://github.com/Kreedzt/rwrs-another-page), developed using Rust and the Salvo framework. Its primary function is to proxy API requests for the Running with Rifles game server list and provide static file serving.

## Development

1. Install Rust development environment

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Or download the installer for Windows
```

2. Clone the repository

```bash
git clone https://github.com/zhaozisong0/rwrs-server.git
cd rwrs-server
```

3. Local development

```bash
# Run the development server
cargo run
```

By default, the server will start on `127.0.0.1:5800`. You can modify this using environment variables:

```bash
# Custom IP and port
HOST=0.0.0.0 PORT=8080 cargo run
```

## Building

### Local Build

```bash
# Build optimized version
cargo build --release
```

After building, the executable will be located at `target/release/rwrs-server`.

### Docker Build

```bash
# Build Docker image
docker build -t rwrs-server .
```

## Deployment

### Direct Deployment

1. Upload the built executable to your server
2. Set environment variables and run

```bash
HOST=0.0.0.0 PORT=80 ./rwrs-server
```

### Docker Deployment

```bash
# Pull the image
docker pull zhaozisong0/rwrs-server:latest

# Run container
docker run -d -p 80:80 --name rwrs-server -e HOST=0.0.0.0 -e PORT=80 zhaozisong0/rwrs-server
```

Alternatively, use docker-compose:

```yaml
version: "3"
services:
  rwrs-server:
    image: zhaozisong0/rwrs-server:latest
    ports:
      - "80:80"
    environment:
      - HOST=0.0.0.0
      - PORT=80
    restart: always
```

## License

- [MIT](./LICENSE).
