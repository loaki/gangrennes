FROM rust:1.88-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd -m appuser
WORKDIR /app

COPY --from=builder /app/target/release/gangrennes /usr/local/bin/gangrennes

RUN mkdir -p /app/data \
    && chown -R appuser:appuser /app

USER appuser

ENV HOST=0.0.0.0 \
    PORT=3000 \
    DATABASE_URL=sqlite:///app/data/app.db?mode=rwc \
    MAX_DB_CONNECTIONS=5 \
    ALLOWED_ORIGIN=* \
    JWT_SECRET=CHANGE_ME_TO_A_LONG_RANDOM_SECRET_AT_LEAST_32_CHARS \
    JWT_EXPIRATION_MINUTES=60 \
    RUST_LOG=info,gangrennes=debug

EXPOSE 3000

CMD ["gangrennes"]