###############################################################################
# match_composer Dockerfile
#
# Builds:
#   1) librcsc            (shared C++ dependency)
#   2) gRPC C++           (needed by SoccerSimulationProxy)
#   3) SoccerSimulationProxy  (Cyrus2D)
#   4) helios-base            (HELIOS)
#   5) match_composer         (Rust binary)
#
# Final image lays out hub/ so that each image dir contains its compiled
# sample_player, player.conf, formations-dt, and start_player.sh side-by-side.
#
# Build from workspace root:
#   docker build -f match_composer/Dockerfile -t match_composer .
###############################################################################

# =============================================================================
# Base build image with common C++ toolchain
# =============================================================================
FROM debian:12-slim AS cpp-base

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
        build-essential automake autoconf libtool flex bison \
        libboost-all-dev cmake pkg-config git wget ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# =============================================================================
# Stage: build librcsc (pinned commit used by SSP README)
# =============================================================================
FROM cpp-base AS librcsc-builder

WORKDIR /tmp
RUN git clone https://github.com/helios-base/librcsc.git && \
    cd librcsc && \
    git checkout 19175f339dcb5c3f61b56a8c1bff5345109f22ef && \
    mkdir build && cd build && \
    cmake .. && \
    make -j"$(nproc)" && \
    make install DESTDIR=/opt/librcsc-install

# Also install to system paths for later stages that build against it
RUN cd /tmp/librcsc/build && make install

# =============================================================================
# Stage: build gRPC C++ v1.62.0
# =============================================================================
FROM cpp-base AS grpc-builder

ENV MY_INSTALL_DIR=/opt/grpc-install

RUN git clone --recurse-submodules -b v1.62.0 --depth 1 --shallow-submodules \
        https://github.com/grpc/grpc /tmp/grpc && \
    mkdir -p /tmp/grpc/cmake/build && \
    cd /tmp/grpc/cmake/build && \
    cmake -DgRPC_INSTALL=ON \
          -DgRPC_BUILD_TESTS=OFF \
          -DCMAKE_INSTALL_PREFIX=${MY_INSTALL_DIR} \
          ../.. && \
    make -j"$(nproc)" && \
    make install

# =============================================================================
# Stage: build SoccerSimulationProxy (Cyrus2D)
# =============================================================================
FROM cpp-base AS ssp-builder

# Bring in librcsc (system install)
COPY --from=librcsc-builder /opt/librcsc-install/ /

# Bring in gRPC
ENV MY_INSTALL_DIR=/opt/grpc-install
COPY --from=grpc-builder ${MY_INSTALL_DIR} ${MY_INSTALL_DIR}
ENV PATH="${MY_INSTALL_DIR}/bin:${PATH}"

WORKDIR /tmp
RUN git clone https://github.com/Cyrus2D/SoccerSimulationProxy.git && \
    cd SoccerSimulationProxy && \
    mkdir build && cd build && \
    cmake -DCMAKE_PREFIX_PATH=${MY_INSTALL_DIR} .. && \
    make -j"$(nproc)"

# After build the artefacts live under /tmp/SoccerSimulationProxy/build/bin/
# which typically contains: sample_player, start.sh, player.conf, formations-dt/, etc.

# =============================================================================
# Stage: build helios-base (HELIOS)
# =============================================================================
FROM cpp-base AS helios-builder

# Bring in librcsc (system install)
COPY --from=librcsc-builder /opt/librcsc-install/ /
RUN ldconfig

WORKDIR /tmp
RUN git clone https://github.com/helios-base/helios-base.git && \
    cd helios-base && \
    git checkout 835e3997bbe60eb76d7dc8828df373b175614fb0 && \
    ./bootstrap && \
    ./configure && \
    make -j"$(nproc)"

# After build the artefacts live under /tmp/helios-base/src/
# which contains: sample_player, player.conf, formations-dt/, etc.

# =============================================================================
# Stage: build the match_composer Rust binary (cargo-chef pattern)
# =============================================================================
FROM rust:slim AS chef
RUN cargo install cargo-chef

FROM chef AS planner
WORKDIR /usr/src/rcss_cluster
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS rust-builder
WORKDIR /usr/src/rcss_cluster

RUN apt update && apt install -y protobuf-compiler

COPY --from=planner /usr/src/rcss_cluster/recipe.json recipe.json
RUN cargo chef cook --release --bin match_composer --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin match_composer

# =============================================================================
# Final runtime image
# =============================================================================
FROM debian:12-slim

LABEL version="0.1.0"
LABEL authors="enricliu"
LABEL description="match_composer with pre-built hub images (SSP + helios-base)"

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
        libboost-system1.81.0 libboost-filesystem1.81.0 \
        libstdc++6 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ---------- match_composer binary ----------
COPY --from=rust-builder /usr/src/rcss_cluster/target/release/match_composer /app/match_composer

# ---------- librcsc shared libs ----------
COPY --from=librcsc-builder /opt/librcsc-install/root/local/lib/ /usr/local/lib/

# ---------- gRPC shared libs (needed by SSP at runtime) ----------
COPY --from=grpc-builder /opt/grpc-install/lib/ /usr/local/lib/

RUN ldconfig

# ---------- hub/Cyrus2D/SoccerSimulationProxy ----------
#   Source build artefacts are in /tmp/SoccerSimulationProxy/build/bin/
#   We place them alongside the custom start_player.sh from our repo.
COPY match_composer/hub/Cyrus2D/SoccerSimulationProxy/start_player.sh \
     /app/hub/Cyrus2D/SoccerSimulationProxy/start_player.sh

COPY --from=ssp-builder /tmp/SoccerSimulationProxy/build/bin/sample_player \
     /app/hub/Cyrus2D/SoccerSimulationProxy/sample_player

COPY --from=ssp-builder /tmp/SoccerSimulationProxy/build/bin/player.conf \
     /app/hub/Cyrus2D/SoccerSimulationProxy/player.conf

COPY --from=ssp-builder /tmp/SoccerSimulationProxy/build/bin/formations-dt \
     /app/hub/Cyrus2D/SoccerSimulationProxy/formations-dt

# ---------- hub/HELIOS/helios-base ----------
#   Source build artefacts are in /tmp/helios-base/src/
#   We place them alongside the custom start_player.sh from our repo.

COPY match_composer/hub/HELIOS/helios-base/start_player.sh \
     /app/hub/HELIOS/helios-base/start_player.sh

COPY --from=helios-builder /tmp/helios-base/src/sample_player \
     /app/hub/HELIOS/helios-base/sample_player

COPY --from=helios-builder /tmp/helios-base/src/player.conf \
     /app/hub/HELIOS/helios-base/player.conf

COPY --from=helios-builder /tmp/helios-base/src/formations-dt \
     /app/hub/HELIOS/helios-base/formations-dt

# Make scripts executable
RUN chmod +x /app/hub/Cyrus2D/SoccerSimulationProxy/start_player.sh \
             /app/hub/HELIOS/helios-base/start_player.sh

ENV HUB_PATH=/app/hub
ENV LOG_ROOT=/app/logs
ENV LD_LIBRARY_PATH=/usr/local/lib

EXPOSE 7777/tcp

ENTRYPOINT ["/app/match_composer"]

