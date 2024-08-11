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

# Install necessary dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && ln -s /usr/lib/x86_64-linux-gnu/libssl.so.3 /usr/lib/x86_64-linux-gnu/libssl.so \
    && ldconfig

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rust-m3u8-proxy /usr/local/bin/rust-m3u8-proxy

# Expose the port
EXPOSE 3030

# Run the application
CMD ["rust-m3u8-proxy"]
