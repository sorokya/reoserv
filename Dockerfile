FROM rust:alpine3.23 AS chef
WORKDIR /app
RUN cargo install --locked cargo-chef
RUN apk add --no-cache musl-dev

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --recipe-path recipe.json --release
COPY . .
RUN cargo build --release

FROM alpine:3.23
WORKDIR /reoserv
COPY --from=builder /app/target/release/reoserv .

EXPOSE 8078

CMD ["./reoserv"]
