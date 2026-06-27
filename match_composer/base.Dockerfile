FROM debian:13-slim AS base

LABEL version="0.7.0"
LABEL authors="enricliu"
LABEL description="nexus-prime Match Composer basic runtime container"


ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    libboost-system-dev libboost-filesystem-dev \
    libstdc++6 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
