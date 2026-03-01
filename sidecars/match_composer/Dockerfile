# =============================================================================
# Stage 1: C++ build base (Ubuntu 22.04 as required by both READMEs)
# =============================================================================
FROM ubuntu:22.04 AS cpp-base

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        automake \
        autoconf \
        libtool \
        flex \
        bison \
        libboost-all-dev \
        cmake \
        git \
        ca-certificates \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

# =============================================================================
# Stage 2: Build gRPC v1.66.0 from source
#   (required by SoccerSimulationProxy per its README)
# =============================================================================
FROM cpp-base AS grpc-builder

ENV MY_INSTALL_DIR=/root/.local
ENV PATH="${MY_INSTALL_DIR}/bin:${PATH}"

RUN mkdir -p "${MY_INSTALL_DIR}"

WORKDIR /build

RUN git clone --recurse-submodules -b v1.66.0 --depth 1 --shallow-submodules \
        https://github.com/grpc/grpc \
    && cd grpc \
    && mkdir -p cmake/build \
    && cd cmake/build \
    && cmake -DgRPC_INSTALL=ON \
             -DgRPC_BUILD_TESTS=OFF \
             -DCMAKE_INSTALL_PREFIX="${MY_INSTALL_DIR}" \
             ../.. \
    && make -j"$(nproc)" \
    && make install

# =============================================================================
# Stage 3: Build librcsc
#   (shared dependency for both helios-base and SoccerSimulationProxy)
# =============================================================================
FROM grpc-builder AS librcsc-builder

WORKDIR /build

RUN git clone https://github.com/helios-base/librcsc.git \
    && cd librcsc \
    && git checkout 19175f339dcb5c3f61b56a8c1bff5345109f22ef \
    && mkdir build && cd build \
    && cmake -DCMAKE_INSTALL_PREFIX="${MY_INSTALL_DIR}" .. \
    && make -j"$(nproc)" \
    && make install

# =============================================================================
# Stage 4: Build helios-base
#   Build method from README:
#     ./bootstrap
#     ./configure
#     make
# =============================================================================
FROM librcsc-builder AS helios-builder

WORKDIR /build

RUN git clone https://github.com/helios-base/helios-base.git \
    && cd helios-base \
    && ./bootstrap \
    && ./configure \
    && make -j"$(nproc)"

# =============================================================================
# Stage 5: Build SoccerSimulationProxy
#   Build method from README:
#     cd SoccerSimulationProxy
#     mkdir build
#     cd build
#     cmake ..
#     make
# =============================================================================
FROM librcsc-builder AS ssp-builder

WORKDIR /build

RUN git clone https://github.com/Cyrus2D/SoccerSimulationProxy.git \
    && cd SoccerSimulationProxy \
    && mkdir build && cd build \
    && cmake -DCMAKE_PREFIX_PATH="${MY_INSTALL_DIR}" .. \
    && make -j"$(nproc)"

# =============================================================================
# Stage 6: Build match_composer (Rust) — using cargo-chef for layer caching
# =============================================================================
FROM rust:alpine AS chef

RUN cargo install cargo-chef

FROM chef AS planner

WORKDIR /usr/src/rcss_cluster
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS rust-builder

WORKDIR /usr/src/rcss_cluster

COPY --from=planner /usr/src/rcss_cluster/recipe.json recipe.json
RUN cargo chef cook --release --bin match_composer --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin match_composer

# =============================================================================
# Final image
# =============================================================================
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
        libboost-system1.74.0 \
        zlib1g \
        libstdc++6 \
        libgcc-s1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# match_composer binary
COPY --from=rust-builder /usr/src/rcss_cluster/target/release/match_composer .

# Hub directory with pre-existing start_player.sh launch scripts
COPY sidecars/match_composer/hub ./hub

# helios-base artifacts — placed at the same level as start_player.sh
# Use src/player/sample_player (the actual binary; src/sample_player is a symlink)
COPY --from=helios-builder /build/helios-base/src/player/sample_player ./hub/HELIOS/helios-base/sample_player
COPY --from=helios-builder /build/helios-base/src/player.conf      ./hub/HELIOS/helios-base/player.conf
COPY --from=helios-builder /build/helios-base/src/formations-dt    ./hub/HELIOS/helios-base/formations-dt

# SoccerSimulationProxy artifacts — placed at the same level as start_player.sh
COPY --from=ssp-builder /build/SoccerSimulationProxy/build/bin/sample_player  ./hub/Cyrus2D/SoccerSimulationProxy/sample_player
COPY --from=ssp-builder /build/SoccerSimulationProxy/build/bin/player.conf    ./hub/Cyrus2D/SoccerSimulationProxy/player.conf
COPY --from=ssp-builder /build/SoccerSimulationProxy/build/bin/coach.conf     ./hub/Cyrus2D/SoccerSimulationProxy/coach.conf
COPY --from=ssp-builder /build/SoccerSimulationProxy/build/bin/formations-dt  ./hub/Cyrus2D/SoccerSimulationProxy/formations-dt

# Runtime shared libraries: librcsc (both teams) + gRPC/protobuf/abseil (SSP)
COPY --from=librcsc-builder /root/.local/lib/librcsc*        /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libgrpc*        /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libprotobuf*    /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libabsl*        /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libupb*         /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libgpr*         /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libre2*         /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libaddress_sorting* /usr/local/lib/
COPY --from=grpc-builder    /root/.local/lib/libcares*       /usr/local/lib/

# Remove static archives and libtool files kept from build stages, then refresh linker cache
RUN find /usr/local/lib -maxdepth 1 \( -name '*.a' -o -name '*.la' \) -delete \
    && ldconfig

ENV HUB_PATH=/app/hub

EXPOSE 6657/tcp

ENTRYPOINT ["./match_composer"]
CMD ["--port", "6657"]
