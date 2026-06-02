# Stage 1: Builder
# FROM rust:1.81-alpine AS builder
FROM rust:alpine AS builder

# Install build dependencies (musl for static linking)
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

WORKDIR /app

# Copy dependency manifests first for caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Now copy the actual source and rebuild
COPY src ./src
RUN touch src/main.rs && cargo build --release

# Strip binary to reduce size
RUN strip target/release/blog_api

# Stage 2: Runtime
FROM alpine:latest

# Install runtime dependencies (ca-certificates for HTTPS, libgcc for musl)
RUN apk add --no-cache ca-certificates libgcc

# Copy the binary from builder
COPY --from=builder /app/target/release/blog_api /usr/local/bin/blog_api

# Create a non-root user to run the app
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser
USER appuser

EXPOSE 8080

CMD ["blog_api"]