FROM rust:alpine AS chef
RUN cargo install cargo-chef
RUN apk add --no-cache protoc protobuf-dev

FROM chef AS planner
WORKDIR /usr/src/rcss_cluster
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /usr/src/rcss_cluster

# Build dependency crates
COPY --from=planner /usr/src/rcss_cluster/recipe.json recipe.json
RUN cargo chef cook --release --package allocator --bin allocator --recipe-path recipe.json

# Build the project
COPY . .
RUN cargo build --release --package allocator --bin allocator


FROM alpine:latest
LABEL version="0.1.0"
LABEL authors="enricliu"
LABEL repository="https://github.com/EnricLiu/rcss_cluster.git"


WORKDIR /usr/local/bin

COPY --from=builder /usr/src/rcss_cluster/target/release/allocator .

EXPOSE 6666/tcp

ENTRYPOINT ["./allocator"]
CMD ["--host", "0.0.0.0", "--http-port", "6666"]
