# Chef stage
FROM rust:1.83-alpine AS chef
USER root
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef
WORKDIR /app

# Planner stage
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM chef AS builder
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json -p allocator
# Build application
COPY . .
RUN cargo build --release -p allocator

# Runtime stage
FROM gcr.io/distroless/static-debian12:nonroot
COPY --from=builder /app/target/release/allocator /usr/local/bin/allocator
ENTRYPOINT ["/usr/local/bin/allocator"]
