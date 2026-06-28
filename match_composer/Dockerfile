###############################################################################
# match_composer Dockerfile
#
# Only builds the Rust binary. The final runtime image is selected with
# BASE_IMAGE:
#   - rcss-mc-base:latest contains runtime dependencies only; provide hub files
#     with an external MC_HUB_PATH mount.
#   - rcss-mc-base:bundled-latest already contains /app/hub.
#
# Build from workspace root:
#   docker build -f match_composer/Dockerfile -t match_composer .
#
# To build against a specific published runtime base:
#   docker build -f match_composer/Dockerfile \
#     --build-arg BASE_IMAGE=registry.cn-beijing.aliyuncs.com/nexus-prime/rcss-mc-base:nightly \
#     -t match_composer .
###############################################################################

ARG BASE_IMAGE=registry.cn-beijing.aliyuncs.com/nexus-prime/rcss-mc-base:latest

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
# Final runtime image
# =============================================================================
FROM ${BASE_IMAGE}

LABEL version="0.1.0"
LABEL authors="enricliu"
LABEL description="match_composer runtime; hub files are resolved via MC_HUB_PATH"

# ---------- match_composer binary ----------
COPY --from=rust-builder /usr/src/rcss_cluster/target/release/match_composer /app/match_composer

ENV MC_LOG_ROOT=/app/logs
ENV MC_HUB_PATH=/app/hub

RUN mkdir -p /app/hub /app/logs

EXPOSE 7777/tcp

ENTRYPOINT ["/app/match_composer"]

