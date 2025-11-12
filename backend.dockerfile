# ===== СТАДИЯ 1: Сборка Rust-бинарника =====
FROM rust:1.91.1-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Копируем Rust-проекты и Cargo.toml из контекста (из корня)
COPY shared ./shared
COPY launcher ./launcher
COPY instance_builder ./instance_builder
COPY Cargo.toml ./Cargo.toml

# Собираем бинарник
WORKDIR /build/instance_builder
RUN cargo build --release -p instance_builder

# ===== СТАДИЯ 2: Финальный Python-контейнер =====
FROM python:3.14-slim

WORKDIR /backend

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY backend/requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY backend/app/ ./app

# Копируем бинарник из builder-стейджа
COPY --from=builder /build/target/release/instance_builder /backend/app/instance_builder/instance_builder

EXPOSE 8000

CMD ["python", "-m", "app.main"]
