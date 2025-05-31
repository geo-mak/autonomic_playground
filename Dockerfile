# Stage 1: Builder
FROM rust:1.87-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY pg_kit/ pg_kit/
COPY client/ client/
COPY server/ server/

WORKDIR /app/server
RUN cargo build --release -p api_server

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/api_server /app/api_server

WORKDIR /app
EXPOSE 8000

CMD ["./api_server"]