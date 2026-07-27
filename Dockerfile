# ── Stage 1: Build the Dioxus WASM frontend ──────────────────────────────────
FROM rust:1.97-trixie AS builder

ARG DIOXUS_CLI_VERSION=v0.7.9
ARG FISSION_WEB_API_URL=

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    pkg-config \
    libssl-dev \
    unzip \
    && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://dioxus.dev/install.sh \
    | bash -s -- "${DIOXUS_CLI_VERSION}" \
    && rustup target add wasm32-unknown-unknown

WORKDIR /build

COPY Cargo.toml Cargo.lock Dioxus.toml index.html ./
COPY src/ ./src/
COPY assets/ ./assets/

# An empty base URL makes the WASM client call /api on its current origin.
ENV FISSION_WEB_API_URL=${FISSION_WEB_API_URL}

RUN dx build --platform web --release

# ── Stage 2: Static frontend + private Railway API gateway ───────────────────
FROM nginx:1.28-alpine

COPY deploy/nginx/default.conf.template /etc/nginx/templates/default.conf.template
COPY --from=builder /build/target/dx/fission-web/release/web/public/ /usr/share/nginx/html/

ENV PORT=8080
ENV FISSION_BACKEND_HOST=fission-backend.railway.internal
ENV FISSION_BACKEND_PORT=7331
ENV NGINX_ENVSUBST_FILTER=^(PORT|FISSION_BACKEND_HOST|FISSION_BACKEND_PORT)$

EXPOSE 8080

