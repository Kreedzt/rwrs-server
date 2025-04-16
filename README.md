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

1. Build the server:
   ```bash
   cargo build --release
   ```

2. Upload the built executable (`target/release/rwrs-server`) to your server

3. Create a `static` directory in the same location as the executable:
   ```bash
   mkdir static
   ```

4. Copy the rwrs-another-page frontend build files (from the `dist` directory) to the `static` directory:
   ```bash
   # Assuming you have the rwrs-another-page frontend build in ./dist
   cp -r /path/to/rwrs-another-page/dist/* ./static/
   ```

5. Set environment variables and run:
   ```bash
   HOST=0.0.0.0 PORT=80 ./target/release/rwrs-server
   ```

### Docker Deployment

There are two ways to deploy using Docker:

#### Option 1: Separate Containers with Network

This approach runs the backend (rwrs-server) and frontend (rwrs-another-page) in separate containers.

1. Create a Docker network:
   ```bash
   docker network create rwrs-network
   ```

2. Start the rwrs-server container:
   ```bash
   docker pull zhaozisong0/rwrs-server:latest
   docker run -d --name rwrs-server --network rwrs-network -e HOST=0.0.0.0 -e PORT=80 zhaozisong0/rwrs-server:latest
   ```

3. Start the rwrs-another-page container:
   ```bash
   docker pull zhaozisong0/rwrs-another-page:latest
   docker run -d --name rwrs-another-page --network rwrs-network -p 80:80 zhaozisong0/rwrs-another-page:latest
   ```

4. Configure a reverse proxy (like Nginx) to route:
   - `/api/*` requests to the rwrs-server container
   - `/*` requests to the rwrs-another-page container

#### Option 2: Single Container

You can also deploy by serving the frontend directly from the rwrs-server's static directory:

1. Pull and run the rwrs-server container:
   ```bash
   docker pull zhaozisong0/rwrs-server:latest
   docker run -d -p 80:80 --name rwrs-server -e HOST=0.0.0.0 -e PORT=80 zhaozisong0/rwrs-server:latest
   ```

2. Copy the frontend build to the container's static directory:
   ```bash
   # Assuming you have the rwrs-another-page frontend build in ./dist
   docker cp ./dist/. rwrs-server:/static/
   ```

   Or mount the directory when starting the container:
   ```bash
   docker run -d -p 80:80 --name rwrs-server -e HOST=0.0.0.0 -e PORT=80 -v $(pwd)/dist:/static zhaozisong0/rwrs-server:latest
   ```

#### Using Docker Compose

Alternatively, use docker-compose:

```yaml
version: '3'
services:
  rwrs-server:
    image: zhaozisong0/rwrs-server:latest
    ports:
      - "80:80"
    environment:
      - HOST=0.0.0.0
      - PORT=80
    restart: always
    # Optional: Mount frontend build to static directory
    # volumes:
    #   - ./dist:/static
```

## License

- [MIT](./LICENSE).
