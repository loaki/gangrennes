FROM rust:1.88.0-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml rust-toolchain.toml ./
COPY src ./src
COPY migrations ./migrations
COPY templates ./templates

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/gangrennes /usr/local/bin/gangrennes

RUN mkdir -p /app/data

ENV BIND_ADDR=0.0.0.0:3000
ENV DATABASE_URL=sqlite:///app/data/gangrennes.sqlite3

EXPOSE 3000

CMD ["gangrennes"]
