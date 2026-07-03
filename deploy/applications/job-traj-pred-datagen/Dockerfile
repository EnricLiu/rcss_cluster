FROM rust:alpine AS chef

RUN apk add --no-cache build-base ca-certificates musl-dev openssl-dev pkgconfig
RUN cargo install cargo-chef --locked

WORKDIR /usr/src/rcss_cluster/deploy/applications/job-traj-pred-datagen

FROM chef AS planner

COPY deploy/applications/job-traj-pred-datagen/Cargo.toml ./
COPY deploy/applications/job-traj-pred-datagen/Cargo.lock ./
COPY deploy/applications/job-traj-pred-datagen/src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /usr/src/rcss_cluster/deploy/applications/job-traj-pred-datagen/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY deploy/applications/job-traj-pred-datagen/Cargo.toml ./
COPY deploy/applications/job-traj-pred-datagen/Cargo.lock ./
COPY deploy/applications/job-traj-pred-datagen/src ./src
RUN cargo build --release --bin job-traj-pred-datagen

FROM alpine:latest

LABEL version="0.1.0"
LABEL authors="enricliu"
LABEL repository="https://github.com/EnricLiu/rcss_cluster.git"

WORKDIR /usr/local/bin
RUN apk add --no-cache ca-certificates libstdc++ openssl

COPY --from=builder /usr/src/rcss_cluster/deploy/applications/job-traj-pred-datagen/target/release/job-traj-pred-datagen .

ENTRYPOINT ["./job-traj-pred-datagen"]
