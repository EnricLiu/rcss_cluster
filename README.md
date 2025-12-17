# rcss_cluster

A Rust-based cluster management system for RoboCup Soccer Simulator (rcssserver) with Agones integration for Kubernetes game server orchestration.

> **⚠️ Work in Progress**: This project is currently under active development. Agones and Docker integrations are not yet available.

## Overview

`rcss_cluster` provides infrastructure for running and managing multiple RoboCup Soccer Simulator instances in a distributed environment. It consists of five main components:

- **client**: API gateway / proxy server for managing rooms and routing client connections to backend servers
- **server**: Backend server providing HTTP/WebSocket API for controlling rcssserver instances
- **service**: Service layer with standalone and Agones deployment modes
- **process**: Process management for spawning and controlling rcssserver with trainer/coach support
- **common**: Shared library containing client utilities, command structures, UDP communication, and common types

## Project Structure

```
rcss_cluster/
├── client/        # API gateway/proxy server (Axum-based, listens on 0.0.0.0:6000)
├── server/        # Backend server (HTTP/WebSocket API, listens on 0.0.0.0:55555)
├── service/       # Service layer (standalone/agones modes)
├── process/       # rcssserver process management with trainer/coach
├── common/        # Shared library (clients, commands, types, UDP)
├── Cargo.toml     # Workspace configuration
├── Dockerfile     # Docker build configuration
└── LICENSE        # MIT License
```

## Requirements

- Rust (Edition 2024)
- Linux (Windows is not currently supported)
- [rcssserver](https://github.com/rcsoccersim/rcssserver) installed
- [Agones](https://agones.dev/) (optional, for Kubernetes deployment)

## Building

```bash
cargo build
```

To build in release mode:

```bash
cargo build --release
```

### Feature Flags

The `server` crate supports different deployment modes via feature flags:

```bash
# Build standalone server (default single-instance mode)
cargo build -p server --features standalone

# Build with Agones integration for Kubernetes
cargo build -p server --features agones
```

> **Note**: `standalone` and `agones` features are mutually exclusive.

## Components

### Client (API Gateway)

The client acts as an API gateway/proxy server for managing rooms and routing connections. By default, it listens on `0.0.0.0:6000`.

Features:
- Room management (`/rooms` endpoints)
- Health check endpoints (`/health`)
- WebSocket proxy connections to backend servers
- Agones allocator integration for room allocation

### Server (Backend)

The backend server provides HTTP and WebSocket endpoints for controlling rcssserver instances. By default, it listens on `0.0.0.0:55555`.

Features:
- HTTP API for trainer commands (`/command`, `/control`, `/gateway`)
- WebSocket API for player connections (`/player`)
- Service status tracking (Uninitialized, Idle, Simulating, Finished)

### Service Layer

The service layer provides abstractions for different deployment modes:

- **StandaloneService**: Single-instance mode for local development and testing
- **AgonesService**: Kubernetes-native game server management (planned)

### Process Management

The `process` crate handles rcssserver lifecycle:

- Process spawning with configurable ports
- Trainer/coach client management (`OfflineCoach`)
- Command execution (trainer commands)
- Status monitoring via watch channels

### Common Library

Shared functionality including:

- Client communication utilities (`client` module)
- Command encoding/decoding (`command` module - trainer and player commands)
- UDP communication (`udp` module)
- Common types (`types` module - play modes, ball position, etc.)

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────────┐
│   Client    │────▶│   Server    │────▶│   rcssserver    │
│ (Gateway)   │     │  (Backend)  │     │   (Process)     │
│  :6000      │     │   :55555    │     │                 │
└─────────────┘     └─────────────┘     └─────────────────┘
       │                   │
       │                   ▼
       │            ┌─────────────┐
       │            │   Service   │
       │            │ (standalone │
       │            │  /agones)   │
       │            └─────────────┘
       │
       ▼
┌─────────────┐
│   Agones    │
│ Allocator   │
└─────────────┘
```

## License

This project is primarily licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

**Note**: The `gameserver.yaml` file is derived from [googleforgames/agones](https://github.com/googleforgames/agones/blob/main/examples/simple-game-server/gameserver.yaml)
and is licensed under Apache License 2.0. The original copyright notice
and license header have been preserved in that file.

## Author

Copyright (c) 2025 EnricLiu
