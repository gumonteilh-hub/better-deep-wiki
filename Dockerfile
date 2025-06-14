# Multi-stage build for optimal image size
FROM rust:1.87-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code and build files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release --features api --bin better-deep-wiki-api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    locales \
    && rm -rf /var/lib/apt/lists/*

# Configure UTF-8 locale
RUN sed -i '/en_US.UTF-8/s/^# //g' /etc/locale.gen && \
    locale-gen
ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

# Create app user for security
RUN useradd -r -s /bin/false better-deep-wiki

# Set working directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/better-deep-wiki-api ./better-deep-wiki-api

# Create necessary directories
RUN mkdir -p clone generated && \
    chown -R better-deep-wiki:better-deep-wiki /app

# Switch to app user
USER better-deep-wiki

# Expose port
EXPOSE 3000


# Run the application
CMD ["./better-deep-wiki-api"]