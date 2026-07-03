FROM rust:alpine AS builder

WORKDIR /usr/src/rcss_cluster
RUN apk add --no-cache build-base ca-certificates musl-dev openssl-dev pkgconfig

COPY deploy/applications/job-traj-pred-datagen ./deploy/applications/job-traj-pred-datagen
RUN cargo build --release \
    --manifest-path deploy/applications/job-traj-pred-datagen/Cargo.toml

FROM alpine:latest

LABEL version="0.1.0"
LABEL authors="enricliu"
LABEL repository="https://github.com/EnricLiu/rcss_cluster.git"

WORKDIR /usr/local/bin
RUN apk add --no-cache ca-certificates libstdc++ openssl

COPY --from=builder /usr/src/rcss_cluster/deploy/applications/job-traj-pred-datagen/target/release/job-traj-pred-datagen .

ENTRYPOINT ["./job-traj-pred-datagen"]
