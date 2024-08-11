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