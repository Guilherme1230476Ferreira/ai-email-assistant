# ── Stage 1: Builder ──────────────────────────────────────────────────────────
# Uses the official Rust image to compile a release binary.
# SQLX_OFFLINE=true uses the committed .sqlx query cache so no live DB needed.
FROM rust:1.80-slim-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY .sqlx ./.sqlx
COPY migrations ./migrations

ENV SQLX_OFFLINE=true
RUN cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
# Minimal debian image — only contains the compiled binary and migrations.
FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ai-email-assistant .
COPY --from=builder /app/migrations ./migrations

EXPOSE 3000

CMD ["./ai-email-assistant"]
