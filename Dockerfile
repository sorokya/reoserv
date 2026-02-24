FROM rust:alpine3.23@sha256:4fec02de605563c297c78a31064c8335bc004fa2b0bf406b1b99441da64e2d2d AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev openssl-dev && \
    cargo install --locked cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release
COPY . .
RUN cargo build --release

FROM alpine:3.23@sha256:25109184c71bdad752c8312a8623239686a9a2071e8825f20acb8f2198c3f659

EXPOSE 8078
EXPOSE 8079

RUN addgroup --gid 1000 reoserv && \
    adduser \
        -D \
        -g "" \
        -h /home/reoserv \
        -u 1000 \
        -G reoserv \
        reoserv

WORKDIR /reoserv

COPY --from=builder /app/target/release/reoserv ./
COPY README.md LICENSE.txt ./

RUN chown -R reoserv:reoserv /reoserv

USER reoserv

CMD ["./reoserv"]
