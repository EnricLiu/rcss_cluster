###############################################################################
# match_composer Dockerfile
#
# Only builds the Rust binary. Requires mc-hub:latest as the base image
# (built separately via Dockerfile.hub, which contains all C++ deps).
#
# Build from workspace root:
#   docker build -f match_composer/Dockerfile -t match_composer .
#
# If the hub base image doesn't exist yet, build it first:
#   docker build -f match_composer/Dockerfile.hub -t mc-hub:latest .
###############################################################################

# =============================================================================
# Stage: cargo-chef planner (extracts dependency recipe)
# =============================================================================
FROM rust:slim-bookworm AS chef
RUN cargo install cargo-chef

FROM chef AS planner
WORKDIR /usr/src/rcss_cluster
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage: build the match_composer Rust binary
# =============================================================================
FROM chef AS rust-builder
WORKDIR /usr/src/rcss_cluster

RUN apt update && apt install -y protobuf-compiler

COPY --from=planner /usr/src/rcss_cluster/recipe.json recipe.json
RUN cargo chef cook --release --bin match_composer --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin match_composer

# =============================================================================
# Final runtime image — based on the pre-built hub image
# =============================================================================
FROM registry.cn-beijing.aliyuncs.com/nexus-prime/rcss-mc-base:latest

LABEL version="0.1.0"
LABEL authors="enricliu"
LABEL description="match_composer with pre-built hub images (SSP + helios-base)"

# ---------- match_composer binary ----------
COPY --from=rust-builder /usr/src/rcss_cluster/target/release/match_composer /app/match_composer

ENV MC_LOG_ROOT=/app/logs

EXPOSE 7777/tcp

ENTRYPOINT ["/app/match_composer"]

