# Build stage
FROM rust:1.83-alpine AS builder

WORKDIR /app

# Install build dependencies (openssl for cloud storage, musl-dev for Rust)
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

# Copy source files
COPY Cargo.toml ./
COPY src ./src

# Build the application with static linking
ENV OPENSSL_STATIC=1
RUN cargo build --release

# Runtime stage
FROM alpine:latest

WORKDIR /app

# Install CA certificates for HTTPS (needed for cloud storage)
RUN apk add --no-cache ca-certificates

# Copy the binary from builder
COPY --from=builder /app/target/release/cimishi .

# Create input/output directories
RUN mkdir -p /app/examples/data /app/examples/queries /app/output

ENTRYPOINT ["./cimishi"]
