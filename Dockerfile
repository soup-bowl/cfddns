# Build stage
FROM rust:1.83-alpine AS builder

WORKDIR /build

# Install build dependencies
RUN apk add --no-cache musl-dev

# Copy manifests
COPY Cargo.toml .

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src src

# Build the application
RUN touch src/main.rs && \
    cargo build --release

# Runtime stage
FROM alpine:latest

# Copy the binary from the builder
COPY --from=builder /build/target/release/cddns /bin/cddns

ENTRYPOINT [ "cddns" ]
