# Build stage
FROM rust:1.83 AS builder

WORKDIR /build

# Copy all source code
COPY Cargo.toml .
COPY src src

# Build the application
RUN cargo build --release && \
    strip target/release/cddns

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder
COPY --from=builder /build/target/release/cddns /bin/cddns

ENTRYPOINT [ "cddns" ]
