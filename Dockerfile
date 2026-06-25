# syntax=docker/dockerfile:1

FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --locked --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --uid 10001 --create-home --shell /usr/sbin/nologin guixu

COPY --from=builder /app/target/release/guixu /usr/local/bin/guixu

USER guixu

ENV PORT=3000
ENV RUST_LOG=guixu=info

EXPOSE 3000

CMD ["guixu"]
