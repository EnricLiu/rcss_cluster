# RoboCup Soccer Server - Observability & Monitoring

Comprehensive monitoring and observability solution for the RoboCup Soccer Server, providing real-time insights into player connections, proxy performance, and rcssserver health.

## Features

### Metrics Collected

#### Player Connection Metrics
- **rcss_player_connections_total**: Total player connections by protocol (websocket/udp)
- **rcss_player_disconnections_total**: Total disconnections by protocol and reason
- **rcss_active_player_sessions**: Current active player sessions by protocol
- **rcss_player_connection_duration_seconds**: Connection duration histogram

#### Proxy Metrics
- **rcss_proxy_messages_sent_total**: Messages sent through proxy by protocol and direction
- **rcss_proxy_messages_received_total**: Messages received through proxy
- **rcss_proxy_message_size_bytes**: Message size distribution histogram
- **rcss_proxy_message_latency_seconds**: Message processing latency histogram
- **rcss_proxy_errors_total**: Proxy errors by protocol and error type

#### Session Metrics
- **rcss_session_created_total**: Total sessions created by protocol
- **rcss_session_timeout_total**: Total session timeouts by protocol
- **rcss_session_reused_total**: Total session reuses by protocol
- **rcss_active_sessions**: Current active sessions by protocol

#### rcssserver Process Metrics
- **rcss_server_status**: Server process status (0=dead, 1=init, 2=booting, 3=running, 4=returned)
- **rcss_server_restarts_total**: Total server restarts
- **rcss_server_uptime_seconds**: Server uptime in seconds
- **rcss_game_timestep**: Current game timestep
- **rcss_game_status**: Game status (0=uninitialized, 1=idle, 2=simulating, 3=finished, 4=shutdown)

#### HTTP Server Metrics
- **rcss_http_requests_total**: Total HTTP requests by method, path, and status
- **rcss_http_request_duration_seconds**: HTTP request duration histogram
- **rcss_websocket_upgrades_total**: Successful WebSocket upgrades
- **rcss_websocket_upgrade_failures_total**: Failed WebSocket upgrades

## Quick Start

### 1. Build the Server with Metrics

The metrics are already integrated into the server code. Build the server:

```bash
cargo build --release --features standalone
```

### 2. Start the Monitoring Stack

```bash
cd monitoring
docker-compose up -d
```

This starts:
- **Prometheus** on port 9090
- **Grafana** on port 3000
- **Alertmanager** on port 9093
- **Node Exporter** on port 9100

### 3. Start the RCSS Server

```bash
./target/release/standalone-server --ip 0.0.0.0 --port 55555
```

### 4. Access the Dashboards

- **Grafana**: http://localhost:3000 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Alertmanager**: http://localhost:9093

### 5. View Metrics Endpoint

The server exposes metrics at:
```
http://localhost:55555/metrics
```

## Grafana Dashboard

The pre-configured dashboard includes:

1. **Player Connection Overview**
   - Active sessions by protocol
   - Connection/disconnection rates
   - Average connection duration

2. **Proxy Performance**
   - Message throughput (sent/received)
   - Message latency percentiles (p95, p99)
   - Message size distribution
   - Error rates

3. **Session Management**
   - Session creation/timeout rates
   - Session reuse statistics
   - Total active sessions

4. **rcssserver Health**
   - Process status indicator
   - Uptime tracking
   - Game timestep progression
   - Game status indicator

5. **WebSocket Metrics**
   - Upgrade success rate
   - Upgrade failures

## Alerts

Pre-configured alerts include:

### Critical Alerts
- **RCSSServerDown**: rcssserver process not running for 1 minute
- **HighProxyErrorRate**: Proxy error rate > 0.5/sec for 5 minutes

### Warning Alerts
- **HighPlayerDisconnectionRate**: Abnormal disconnection rate
- **HighProxyLatency**: p95 latency > 100ms for 5 minutes
- **SessionTimeoutSpike**: High session timeout rate
- **RCSSServerFrequentRestarts**: Server restarting frequently
- **GameStuck**: Game timestep not advancing while simulating
- **HighWebSocketUpgradeFailureRate**: >10% WebSocket upgrade failures

### Info Alerts
- **NoActivePlayerSessions**: No players connected for 10 minutes

## Configuration

### Prometheus

Edit `monitoring/prometheus/prometheus.yml` to configure:
- Scrape intervals
- Target endpoints
- External labels

### Alertmanager

Edit `monitoring/alertmanager/alertmanager.yml` to configure:
- Slack webhook URL
- Alert routing rules
- Notification channels

Set environment variable for Slack:
```bash
export SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
```

### Grafana

Default credentials:
- Username: `admin`
- Password: `admin`

Change on first login or set via environment variables in `docker-compose.yml`:
```yaml
environment:
  - GF_SECURITY_ADMIN_PASSWORD=your_secure_password
```

## Metrics Architecture

```
┌─────────────────┐
│  RCSS Server    │
│  (Port 55555)   │
│                 │
│  /metrics       │◄─────┐
└─────────────────┘      │
                         │ Scrape every 10s
┌─────────────────┐      │
│  Prometheus     │──────┘
│  (Port 9090)    │
│                 │
│  - Stores       │
│  - Queries      │
│  - Alerts       │
└────────┬────────┘
         │
         │ Data Source
         │
┌────────▼────────┐
│  Grafana        │
│  (Port 3000)    │
│                 │
│  - Dashboards   │
│  - Visualization│
└─────────────────┘

┌─────────────────┐
│  Alertmanager   │◄──── Alert Rules
│  (Port 9093)    │
│                 │
│  - Slack        │
│  - Webhooks     │
└─────────────────┘
```

## Query Examples

### Player Connection Rate (5m)
```promql
rate(rcss_player_connections_total[5m])
```

### Proxy Latency p95
```promql
histogram_quantile(0.95,
  sum(rate(rcss_proxy_message_latency_seconds_bucket[5m])) by (le, protocol)
)
```

### Active Sessions by Protocol
```promql
rcss_active_player_sessions
```

### Error Rate by Type
```promql
rate(rcss_proxy_errors_total[5m])
```

### Average Connection Duration
```promql
rate(rcss_player_connection_duration_seconds_sum[5m]) /
rate(rcss_player_connection_duration_seconds_count[5m])
```

## Troubleshooting

### Metrics Not Appearing

1. Check server is running:
```bash
curl http://localhost:55555/metrics
```

2. Check Prometheus targets:
```
http://localhost:9090/targets
```

3. Verify Prometheus can reach the server:
```bash
docker exec rcss-prometheus wget -O- http://host.docker.internal:55555/metrics
```

### Grafana Dashboard Empty

1. Verify Prometheus datasource is configured
2. Check time range in dashboard (default: last 1 hour)
3. Ensure server has been running and receiving traffic

### Alerts Not Firing

1. Check alert rules in Prometheus:
```
http://localhost:9090/alerts
```

2. Verify Alertmanager is receiving alerts:
```
http://localhost:9093/#/alerts
```

3. Check Alertmanager logs:
```bash
docker logs rcss-alertmanager
```

## Production Deployment

### Security Considerations

1. **Change default passwords**:
   - Grafana admin password
   - Add authentication to Prometheus

2. **Use TLS**:
   - Configure HTTPS for Grafana
   - Use TLS for Prometheus scraping

3. **Network isolation**:
   - Use Docker networks
   - Firewall rules for metric endpoints

4. **Access control**:
   - Restrict /metrics endpoint
   - Use Prometheus authentication

### Scaling

For high-traffic deployments:

1. **Increase Prometheus retention**:
```yaml
command:
  - '--storage.tsdb.retention.time=90d'
```

2. **Use remote storage**:
   - Thanos
   - Cortex
   - VictoriaMetrics

3. **Shard Prometheus**:
   - Multiple Prometheus instances
   - Federation for aggregation

## Integration with Existing Tools

### Export to Other Systems

Prometheus supports remote write to:
- InfluxDB
- Elasticsearch
- Datadog
- New Relic

### Custom Exporters

Add custom metrics in your code:

```rust
use crate::metrics::collector::METRICS_COLLECTOR;

// Increment a counter
METRICS_COLLECTOR.player_connections_total
    .with_label_values(&["websocket"])
    .inc();

// Observe a histogram
METRICS_COLLECTOR.proxy_message_latency
    .with_label_values(&["udp"])
    .observe(latency_seconds);

// Set a gauge
METRICS_COLLECTOR.active_player_sessions
    .with_label_values(&["websocket"])
    .set(count as f64);
```

## License

Same as the main project.
