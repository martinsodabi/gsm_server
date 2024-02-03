FROM rust:1.75.0-buster as builder

USER root
RUN apt-get install -y --no-install-recommends ca-certificates
RUN update-ca-certificates

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/date-tv ./date-tv

# Runtime image
FROM ubuntu:20.04

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/date-tv /app/date-tv

# Run the app
CMD ./date-tv
