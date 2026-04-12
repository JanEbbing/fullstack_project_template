# Stage 1: Build frontend
FROM node:22-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# Stage 2: Build backend
FROM rust:1.84-slim AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY backend/Cargo.toml backend/Cargo.lock ./
# Dependency caching: build with dummy main.rs first
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs
RUN cargo build --release
RUN rm -rf src
COPY backend/src ./src
RUN touch src/main.rs src/lib.rs
RUN cargo build --release

# Stage 3: Final slim image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN useradd --system --uid 1001 --no-create-home appuser
WORKDIR /app
COPY --from=backend-builder /app/target/release/fullstack-app ./
COPY --from=frontend-builder /app/frontend/build ./static/
RUN mkdir -p /app/data && chown appuser /app/data
USER 1001

ENV HOST=0.0.0.0
ENV PORT=3000
ENV STATIC_DIR=./static
ENV DATABASE_URL=/app/data/app.db

EXPOSE 3000
CMD ["./fullstack-app"]
