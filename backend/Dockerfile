FROM rust:1.91-slim-bookworm AS instance-builder
WORKDIR /src

COPY shared ./shared
COPY launcher ./launcher
COPY instance_builder ./instance_builder
COPY Cargo.toml Cargo.lock ./

RUN cargo build --locked --release -p instance_builder


FROM golang:1.25-bookworm AS backend-builder
WORKDIR /src/backend

COPY backend/go.mod backend/go.sum ./
RUN go mod download

COPY backend/ .
RUN CGO_ENABLED=0 GOOS=linux GOARCH=amd64 go build -o /out/server ./cmd/server


FROM debian:bookworm-slim AS runtime
ENV SPEC_FILE=/data/metadata/spec.json \
    GENERATED_DIR=/data/generated \
    WORKDIR_DIR=/data/workdir \
    UPLOADED_INSTANCES_DIR=/data/uploaded-instances \
    TEMP_DIR=/tmp/potato \
    HOST=0.0.0.0 \
    PORT=8000

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /out/server /usr/local/bin/server
COPY --from=instance-builder /src/target/release/instance_builder /usr/local/bin/instance_builder

RUN mkdir -p /data/uploaded-instances /data/generated /data/workdir /data/metadata

EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/server"]
