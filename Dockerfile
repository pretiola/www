# Builder stage
FROM rust:1-slim-bookworm as builder

WORKDIR /app
# Install dependencies required by some crates
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy over manifests
COPY Cargo.toml Cargo.lock ./

# Cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source tree
COPY . .

# Build for release
RUN touch src/main.rs && cargo build --release

# Asset build stage (CSS + optimized images)
FROM node:20-slim as asset-builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install
COPY templates templates
COPY static static
COPY scripts scripts
RUN npm run build:css
RUN npm run build:images

# Runtime stage
FROM debian:bookworm-slim

# Install root certs and libssl
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/pretiola /usr/local/bin/

# Copy standard setup
COPY static /app/static
COPY templates /app/templates
# Override with compiled css and optimized images from asset-builder
COPY --from=asset-builder /app/static/css/tailwind.css /app/static/css/tailwind.css
COPY --from=asset-builder /app/static/pictures/optimized/ /app/static/pictures/optimized/

ENV RUST_LOG="info"
EXPOSE 8080

CMD ["pretiola"]
