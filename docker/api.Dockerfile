# syntax=docker/dockerfile:1
# Build from repo root:
#   docker build -f docker/api.Dockerfile -t panagang-api .

FROM rust:1.97-bookworm AS builder
WORKDIR /app
COPY apps/api ./
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/panagang_api /usr/local/bin/panagang_api
ENV API_HOST=0.0.0.0
ENV API_PORT=8080
EXPOSE 8080
CMD ["panagang_api"]
