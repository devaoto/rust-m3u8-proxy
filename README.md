# M3U8 CORS Proxy

This is a simple CORS proxy built with Rust and Warp. It allows you to forward requests to a specified URL while handling CORS headers.

## Features

- Handles GET requests and proxies them to the specified target URL.
- Supports adding `Referer` and `Origin` headers.
- Optionally proxies all URLs in `.m3u8` playlists.

## Requirements

- Rust (latest stable version)
- Cargo (comes with Rust)

## Setup

1. **Clone the repository:**

   ```bash
   git clone https://github.com/devaoto/rust-m3u8-proxy.git
   cd rust-m3u8-proxy
   ```

2. **Build the project:**

   ```bash
   cargo build --release
   ```

3. **Run the application:**

   ```bash
   cargo run --release
   ```

   The server will start on `http://127.0.0.1:3030`.

## Usage

Make a GET request to the `/proxy` endpoint with the following query parameters:

- `url`: The target URL to fetch.
- `referer`: (Optional) The Referer header value.
- `origin`: (Optional) The Origin header value.
- `all`: (Optional) Set to `yes` to proxy all HTTP links in `.m3u8` playlists.

Example:

```bash
curl "http://127.0.0.1:3030/proxy?url=https://example.com/stream.m3u8&referer=https://yourreferer.com&origin=https://yourorigin.com&all=yes"
```

#### Note: All the links must be URI Encoded

## Hosting

You can host this application using services like:

- **Railway**: [railway.app](https://railway.app/) (Paid)
- **Render**: [render.com](https://render.com/) (Free)
- **Heroku**: [heroku.com](https://www.heroku.com/) (Half-paid, CC required)

## Docker Support

You can run this application in a Docker container. See the `Dockerfile` section below for instructions.

## Contributing

Feel free to open issues or submit pull requests to enhance the functionality of this CORS proxy.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Dockerfile

```dockerfile
# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Use a minimal base image for the final image
FROM debian:buster-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rust-m3u8-proxy /usr/local/bin/rust-m3u8-proxy

# Expose the port
EXPOSE 3030

# Run the application
CMD ["rust-m3u8-proxy"]
```

### Running with Docker

1. **Build the Docker image:**

   ```bash
   docker build -t rust-m3u8-proxy .
   ```

2. **Run the Docker container:**

   ```bash
   docker run -p 3030:3030 rust-m3u8-proxy
   ```

Your application should now be accessible at `http://localhost:3030`.

```

```
