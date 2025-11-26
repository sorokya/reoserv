# Builder
FROM rust:1.91-trixie as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new reoserv

# We want dependencies cached, so copy those first.
COPY Cargo.toml Cargo.lock /usr/src/reoserv/

# Set the working directory
WORKDIR /usr/src/reoserv

# Install target platform (Cross-Compilation) --> Needed for Alpine
RUN apt update && apt install -y musl-tools musl-dev && \
    update-ca-certificates && \
    rustup target add x86_64-unknown-linux-musl && \
    rustup component add rustfmt

# Copy cargo config
COPY .cargo /usr/src/reoserv/.cargo/

# This is a dummy build to get the dependencies cached.
RUN cargo build --target x86_64-unknown-linux-musl --release --features=console

# Now copy in the rest of the sources
COPY src /usr/src/reoserv/src/

# Touch main.rs to prevent cached release build
RUN touch /usr/src/reoserv/src/main.rs

# This is the actual application build.
RUN cargo build --target x86_64-unknown-linux-musl --release --features=console

# Runtime
FROM alpine:3.22.2 as runtime

# Install netcat-openbsd for better nc behavior than BusyBox's default
RUN apk add --no-cache netcat-openbsd bash

COPY healthcheck.sh /usr/local/bin/healthcheck.sh
RUN chmod +x /usr/local/bin/healthcheck.sh

# Copy application binary from builder image
COPY --from=builder /usr/src/reoserv/target/x86_64-unknown-linux-musl/release/reoserv /usr/bin/

EXPOSE 8078

# Add console port
EXPOSE 6669

WORKDIR /reoserv

# Run the application
CMD ["/usr/bin/reoserv"]

# Add healthcheck
HEALTHCHECK --interval=60s --timeout=5s --start-period=10s --retries=3 \
  CMD /usr/local/bin/healthcheck.sh || exit 1