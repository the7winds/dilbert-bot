FROM rust:1.58-slim AS builder
WORKDIR /repo
COPY . .
RUN apt-get update -y && apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app
RUN apt-get update -y && apt-get install -y --no-install-recommends \
        openssl
COPY --from=builder /repo/target/release/dilbert-bot dilbert-bot
ENTRYPOINT ["./dilbert-bot"]
