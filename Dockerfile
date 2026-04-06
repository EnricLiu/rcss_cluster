FROM rust:alpine AS chef
RUN cargo install cargo-chef
RUN apk add --no-cache protoc protobuf-dev

FROM chef AS planner
WORKDIR /usr/src/rcss_cluster
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /usr/src/rcss_cluster

# Install rcssserver
RUN apk add --no-cache build-base automake autoconf libtool flex-dev bison boost-dev
RUN wget https://github.com/rcsoccersim/rcssserver/releases/download/rcssserver-19.0.0/rcssserver-19.0.0.tar.gz
RUN tar -zvxf rcssserver-19.0.0.tar.gz && \
    cd rcssserver-19.0.0 && \
    ./configure --disable-rcssclient && make -j4 && make install

# Build dependency crates
COPY --from=planner /usr/src/rcss_cluster/recipe.json recipe.json
RUN cargo chef cook --release --bin agones-server --features "agones" --recipe-path recipe.json

# Build the project
COPY . .
RUN cargo build --release --bin agones-server --features "agones"


FROM alpine:latest
LABEL version="0.1.1"
LABEL authors="enricliu"
LABEL repository="https://github.com/EnricLiu/rcss_cluster.git"


WORKDIR /usr/local/bin
ENV LD_LIBRARY_PATH="/usr/local/lib"
RUN apk add --no-cache coreutils libstdc++

COPY --from=builder /usr/src/rcss_cluster/target/release/agones-server .
COPY --from=builder /usr/local/bin/* /usr/local/bin/
COPY --from=builder /usr/local/lib/* /usr/local/lib/

EXPOSE 6000/udp 6001/udp 6002/udp
EXPOSE 6666/tcp

ENTRYPOINT ["./agones-server"]
