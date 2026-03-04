# Allocator Architecture

## Overview

The **Allocator** is a custom Agones GameServer allocator service built with Axum (Rust). It receives allocation requests from the **Client** (proxy), validates them, and creates `GameServerAllocation` CRDs in Kubernetes to claim a ready GameServer from an Agones Fleet.

## Architecture Diagram

```mermaid
flowchart TB
    subgraph External["External Clients"]
        RL["RL Training Client"]
    end

    subgraph ClientService["Client Service (port 6000)"]
        PS["ProxyServer"]
        AC["AgonesClient"]
        RM["Room Manager<br/>(DashMap&lt;String, Room&gt;)"]
        PS --> AC
        PS --> RM
    end

    subgraph AllocatorService["Allocator Service (port 8080)"]
        direction TB

        subgraph HTTPLayer["HTTP Layer (Axum Router)"]
            direction LR
            Health["/health<br/>/ready"]
            AllocEP["POST /api/v1/allocate"]
        end

        subgraph Middleware["Middleware"]
            Auth["auth_middleware<br/>(Optional Bearer Token)"]
        end

        subgraph Controller["Controller Layer"]
            direction TB
            Validate["Request Validation<br/>• bot_count range check<br/>• client_version whitelist"]
            BuildAnno["Build Annotations<br/>• bot_count<br/>• difficulty<br/>• env_params<br/>• client_version"]
            Validate --> BuildAnno
        end

        subgraph ConfigMod["Configuration (clap CLI / ENV)"]
            Config["Config<br/>─────────────────<br/>bind_address: String<br/>fleet_name: String<br/>namespace: String<br/>min/max_bot_count: u32<br/>auth_token: Option&lt;String&gt;<br/>allowed_versions: Vec&lt;String&gt;<br/>scheduling: Scheduling"]
            SchedEnum["Scheduling<br/>─────────────────<br/>Packed | Distributed"]
        end

        subgraph K8sLayer["Kubernetes Client Layer"]
            K8sClient["K8sClient<br/>(kube::Client wrapper)"]
            CreateAlloc["create_allocation()"]
            K8sClient --> CreateAlloc
        end

        subgraph Types["Request / Response Types"]
            direction LR
            AllocReq["AllocateRequest<br/>─────────────────<br/>bot_count: u32<br/>difficulty: Option&lt;String&gt;<br/>env_params: Option&lt;HashMap&gt;<br/>client_version: Option&lt;String&gt;"]
            AllocResp["AllocateResponse<br/>─────────────────<br/>ip: String<br/>port: u16<br/>game_server_name: String"]
        end

        subgraph ErrorTypes["Error Handling"]
            Err["Error Enum<br/>─────────────────<br/>Validation → 400<br/>Auth → 401<br/>K8s → 500<br/>ResourceExhausted → 503<br/>Internal → 500"]
        end

        AllocEP --> Auth
        Auth --> Controller
        Controller --> K8sLayer
    end

    subgraph K8sCluster["Kubernetes Cluster"]
        subgraph Agones["Agones System"]
            GSA_CRD["GameServerAllocation CRD<br/>(allocation.agones.dev/v1)"]
        end
        subgraph Fleet["Agones Fleet: agones-rcss-server"]
            GS1["GameServer #1<br/>Ports: 55555, 6000, 6001, 6002<br/>Counters: rooms<br/>Lists: players"]
            GS2["GameServer #2"]
            GS3["GameServer #N..."]
        end
        GSA_CRD -- "Selects Ready GS<br/>(matchLabels: agones.dev/fleet)" --> Fleet
    end

    subgraph K8sRBAC["RBAC"]
        SA["ServiceAccount: allocator-sa"]
        CR["ClusterRole: allocator-role<br/>verbs: [create]<br/>resources: [gameserverallocations]"]
        SA -.- CR
    end

    RL -- "Connect / Create Room" --> PS
    AC -- "POST /api/v1/allocate<br/>{bot_count, difficulty, ...}" --> AllocEP
    CreateAlloc -- "POST GameServerAllocation<br/>(via kube API)" --> GSA_CRD
    GSA_CRD -- "status: Allocated<br/>{address, ports, gameServerName}" --> CreateAlloc
    CreateAlloc -- "AllocateResponse<br/>{ip, port, game_server_name}" --> AC
    AC -- "WebSocket URL" --> PS
    PS -- "Connect to allocated<br/>GameServer" --> GS1
    SA -.- K8sClient
```

## Allocation Flow (Sequence)

```mermaid
sequenceDiagram
    participant C as RL Client
    participant P as ProxyServer (Client)
    participant A as Allocator Service
    participant K as Kubernetes API
    participant AG as Agones Fleet

    C->>P: Create Room Request
    P->>A: POST /api/v1/allocate<br/>{bot_count, difficulty, env_params, client_version}

    Note over A: Auth Middleware<br/>(optional Bearer token check)

    A->>A: Validate bot_count range
    A->>A: Validate client_version whitelist
    A->>A: Build annotations from request

    A->>K: POST GameServerAllocation CRD<br/>namespace, fleet selector,<br/>scheduling strategy, annotations
    K->>AG: Find Ready GameServer<br/>(label: agones.dev/fleet)
    AG-->>K: Allocate GameServer → "Allocated"
    K-->>A: Status {state, address, ports, gameServerName}

    alt state == "Allocated"
        A-->>P: 200 {ip, port, game_server_name}
        P->>AG: WebSocket connect to GameServer
        P-->>C: Room Created (RoomConfig)
    else state != "Allocated"
        A-->>P: 503 ResourceExhausted
        P-->>C: Error: No servers available
    end
```

## Module Structure

```mermaid
graph LR
    subgraph "allocator/src"
        main["main.rs<br/>─────────────<br/>Entry point<br/>Config parse<br/>K8s client init<br/>Axum server start<br/>Graceful shutdown"]

        config["config.rs<br/>─────────────<br/>Config (clap)<br/>Scheduling enum<br/>Validation logic"]

        auth["auth.rs<br/>─────────────<br/>auth_middleware<br/>Bearer token check"]

        subgraph controller["controller/"]
            mod_ctrl["mod.rs<br/>create_router()"]
            allocate["allocate.rs<br/>─────────────<br/>AppState struct<br/>allocate() handler"]
            health["health.rs<br/>─────────────<br/>health()<br/>ready()"]
            response["response.rs<br/>─────────────<br/>AllocateRequest<br/>AllocateResponse"]
            error["error.rs<br/>─────────────<br/>Error enum<br/>IntoResponse impl"]
        end

        subgraph k8s["k8s/"]
            mod_k8s["mod.rs<br/>K8sClient struct"]
            allocation["allocation.rs<br/>─────────────<br/>GameServerAllocation<br/>GameServerAllocationSpec<br/>GameServerSelector<br/>AllocationMetadata<br/>GameServerAllocationStatus<br/>GameServerPort<br/>create_allocation()"]
        end

        main --> config
        main --> controller
        main --> k8s
        allocate --> config
        allocate --> k8s
        allocate --> response
        allocate --> error
        mod_ctrl --> allocate
        mod_ctrl --> health
    end
```

## Kubernetes Deployment Topology

```mermaid
graph TB
    subgraph ns["Kubernetes Namespace: default"]
        subgraph deploy["Deployment: allocator<br/>(replicas: 2)"]
            Pod1["Pod #1<br/>allocator:8080<br/>CPU: 100m-500m<br/>RAM: 128Mi-256Mi"]
            Pod2["Pod #2<br/>allocator:8080"]
        end
        Svc["Service: allocator<br/>ClusterIP :80 → :8080"]
        SA2["ServiceAccount:<br/>allocator-sa"]

        Svc --> Pod1
        Svc --> Pod2
        SA2 -.- Pod1
        SA2 -.- Pod2
    end

    CRB["ClusterRoleBinding:<br/>allocator-rolebinding"]
    CRole["ClusterRole: allocator-role<br/>allocation.agones.dev/*<br/>verbs: [create]"]
    CRB --> CRole
    SA2 -.- CRB

    subgraph fleet["Fleet: agones-rcss-server<br/>(replicas: 5, Packed)"]
        GS1p["GameServer<br/>Ports:<br/>default:55555<br/>player:6000<br/>trainer:6001<br/>coach:6002"]
        GS2p["GameServer ..."]
    end

    Pod1 -- "create GSA CRD" --> fleet
```

## Key Design Notes

1. **Scheduling Strategies**: Supports `Packed` (fill existing nodes first) and `Distributed` (spread across nodes) via Agones scheduling.
2. **Fleet Selector**: Allocation targets GameServers by label `agones.dev/fleet: <fleet_name>`.
3. **Annotations Passthrough**: Client parameters (`bot_count`, `difficulty`, `env_params`, `client_version`) are passed as annotations on the `GameServerAllocation` metadata for the GameServer to consume.
4. **Auth is Optional**: Bearer token authentication can be enabled via `AUTH_TOKEN` env var; when unset, all requests are allowed.
5. **Version Gating**: `ALLOWED_VERSIONS` enables restricting which client versions can allocate servers.
6. **GameServer Ports**: Each allocated GameServer exposes 4 ports — `default` (HTTP API:55555), `player` (client proxy:6000), `trainer` (6001), and `coach` (6002).
7. **Resource Tracking**: Fleet uses Agones counters (`rooms`) and lists (`players`) for priority-based allocation.
