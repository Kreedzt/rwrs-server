# RWRS Server

Backend server for [rwrs-another-page](https://github.com/Kreedzt/rwrs-another-page).

## Introduction

This project serves as the backend for [rwrs-another-page](https://github.com/Kreedzt/rwrs-another-page), developed using Rust and the Salvo framework. Its primary function is to proxy API requests for the Running with Rifles game server list, provide static file serving, and serve game map configuration data.

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

# Custom cache and rate limiting settings
CACHE_DURATION_SECS=60 RATE_LIMIT_SECS=10 cargo run
```

## Environment Variables

| Variable | Default | Description |
|----------|----------|-------------|
| `HOST` | `127.0.0.1` | Server bind address |
| `PORT` | `5800` | Server port |
| `MAPS_CONFIG` | `maps.json` | Maps configuration file path |
| `CACHE_DURATION_SECS` | `3` | Cache expiry time in seconds |
| `RATE_LIMIT_SECS` | `3` | Rate limit interval in seconds |

## Maps Configuration

The server provides a `/api/maps` endpoint that returns game map configuration data. Maps are configured through a JSON file specified by the `MAPS_CONFIG` environment variable (default: `maps.json`).

### Custom Configuration File Path

You can specify a custom location for the maps configuration file:

```bash
# Use a custom configuration file
MAPS_CONFIG=/etc/rwrs/custom-maps.json cargo run

# In Docker
docker run -e MAPS_CONFIG=/config/maps.json -v $(pwd)/maps.json:/config/maps.json rwrs-server
```

### Maps Configuration File Format

Create a `maps.json` file in the same directory as the executable (or specify a custom path):

```json
{
  "maps": [
    {
      "name": "map9",
      "path": "media/packages/vanilla.desert/maps/map9",
      "image": "md5_1.png"
    },
    {
      "name": "map10",
      "path": "media/packages/vanilla.desert/maps/map10",
      "image": "md5_2.png"
    },
    {
      "name": "map5",
      "path": "media/packages/vanilla.jungle/maps/map5",
      "image": "md5_3.png"
    }
  ]
}
```

#### Map Fields

- **`name`**: Human-readable map name (e.g., "map9")
- **`path`**: Full game path for the map (used as unique identifier)
- **`image`**: Image filename or CDN URL for the map preview

### API Endpoint

**GET** `/api/maps`

Returns a JSON response with all configured maps:

```json
{
  "maps": [
    {
      "name": "map9",
      "path": "media/packages/vanilla.desert/maps/map9",
      "image": "md5_1.png"
    }
  ]
}
```

### Configuration Examples

#### Basic Local Images
```json
{
  "maps": [
    {
      "name": "desert_outpost",
      "path": "media/packages/vanilla.desert/maps/map9",
      "image": "desert_outpost.png"
    }
  ]
}
```

#### CDN Images
```json
{
  "maps": [
    {
      "name": "desert_outpost",
      "path": "media/packages/vanilla.desert/maps/map9",
      "image": "https://cdn.example.com/maps/desert_outpost.png"
    }
  ]
}
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

5. Copy or create the `maps.json` configuration file in the same directory:
   ```bash
   # Copy the example maps.json
   cp maps.json ./deploy/maps.json
   # Or create a custom one based on the examples above
   ```

6. Set environment variables and run:
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
   docker run -d -p 80:80 --name rwrs-server -e HOST=0.0.0.0 -e PORT=80 \
     -v $(pwd)/dist:/static \
     -v $(pwd)/maps.json:/maps.json \
     zhaozisong0/rwrs-server:latest
   ```

#### Using Custom Maps Configuration Path

If you want to use a custom location for the maps configuration:

```bash
docker run -d -p 80:80 --name rwrs-server \
  -e HOST=0.0.0.0 -e PORT=80 \
  -e MAPS_CONFIG=/config/custom-maps.json \
  -v $(pwd)/dist:/static \
  -v $(pwd)/config/custom-maps.json:/config/custom-maps.json \
  zhaozisong0/rwrs-server:latest
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
      - MAPS_CONFIG=/config/maps.json
    restart: always
    volumes:
      # Mount frontend build to static directory
      - ./dist:/static
      # Mount maps configuration
      - ./maps.json:/config/maps.json
```

#### With Custom Maps Configuration

```yaml
version: '3'
services:
  rwrs-server:
    image: zhaozisong0/rwrs-server:latest
    ports:
      - "8080:5800"
    environment:
      - HOST=0.0.0.0
      - PORT=5800
      - MAPS_CONFIG=/app/config/custom-maps.json
      - CACHE_DURATION_SECS=10
    restart: always
    volumes:
      - ./dist:/static
      - ./config/custom-maps.json:/app/config/custom-maps.json
```

## License

- [MIT](./LICENSE).
