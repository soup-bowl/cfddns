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
FROM gcr.io/distroless/cc-debian13:nonroot

COPY --from=builder /build/target/release/cddns /usr/local/bin/cddns

USER nonroot:nonroot
ENTRYPOINT ["/usr/local/bin/cddns"]
