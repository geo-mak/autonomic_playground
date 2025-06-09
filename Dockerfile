# Stage 1: Builder
FROM rust:1.87-slim AS builder

WORKDIR /autonomic_pg

COPY Cargo.toml Cargo.lock ./
COPY pg_kit/ pg_kit/
COPY client/ client/
COPY server/ server/

WORKDIR /autonomic_pg/server
RUN cargo build --release -p api_server

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /autonomic_pg/target/release/api_server /autonomic_pg/api_server

WORKDIR /autonomic_pg
EXPOSE 8000

CMD ["./api_server"]