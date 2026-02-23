FROM --platform=$BUILDPLATFORM rust:alpine3.23 AS chef
WORKDIR /app
ENV PKGCONFIG_SYSROOTDIR=/
RUN apk add --no-cache musl-dev openssl-dev zig && \
    cargo install --locked cargo-zigbuild cargo-chef && \
    rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release --zigbuild \
    --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl
COPY . .
RUN cargo zigbuild --release \
    --target x86_64-unknown-linux-musl \
    --target aarch64-unknown-linux-musl && \
    mkdir -p /app/linux/ && \
    cp target/x86_64-unknown-linux-musl/release/reoserv /app/linux/arm64 && \
    cp target/aarch64-unknown-linux-musl/release/reoserv /app/linux/amd64

FROM alpine:3.23
WORKDIR /reoserv
ARG TARGETPLATFORM
COPY --from=builder /app/${TARGETPLATFORM} ./reoserv
COPY README.md LICENSE.txt ./

EXPOSE 8078

CMD ["./reoserv"]
