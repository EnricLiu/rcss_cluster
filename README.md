# rcss_cluster

A Rust-based cluster management system for RoboCup Soccer Simulator (rcssserver) with Agones integration for Kubernetes game server orchestration.

> **⚠️ Work in Progress**: This project is currently under active development. Agones and Docker integrations are not yet available.

## Overview

`rcss_cluster` provides infrastructure for running and managing multiple RoboCup Soccer Simulator instances in a distributed environment. It consists of three main components:

- **api**: REST/WebSocket API server built with Axum for external communication
- **common**: Shared library containing client utilities, command structures, UDP communication, and common types
- **sidecar**: A sidecar service that manages rcssserver processes and integrates with Agones for Kubernetes-native game server management

## Project Structure

```
rcss_cluster/
├── api/           # API server (Axum-based REST/WebSocket)
├── common/        # Shared library (clients, commands, types)
├── sidecar/       # Sidecar service for rcssserver management
├── Cargo.toml     # Workspace configuration
└── LICENSE        # MIT License
```

## Requirements

- Rust (Edition 2024)
- Linux (Windows is not currently supported)
- [rcssserver](https://github.com/rcsoccersim/rcssserver) installed
- [Agones](https://agones.dev/) (planned for Kubernetes deployment, not yet integrated)

## Building

```bash
cargo build
```

To build in release mode:

```bash
cargo build --release
```

## Components

### API Server

The API server provides HTTP and WebSocket endpoints for interacting with the cluster. By default, it listens on `0.0.0.0:55555`.

### Sidecar

The sidecar manages rcssserver processes. It handles:

- Process spawning and management
- Trainer/coach command execution
- Status tracking (Uninitialized, Idle, Simulating, Finished)

> **Note**: Agones SDK integration for game server lifecycle management is planned but not yet implemented.

### Common Library

Shared functionality including:

- Client communication utilities
- Command encoding/decoding (trainer and player commands)
- UDP communication
- Common types and structures

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Author

Copyright (c) 2025 EnricLiu
