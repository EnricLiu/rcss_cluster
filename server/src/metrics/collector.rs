use lazy_static::lazy_static;
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, Opts, Registry,
};
use std::sync::Arc;

lazy_static! {
    pub static ref METRICS_COLLECTOR: Arc<MetricsCollector> = Arc::new(MetricsCollector::new());
}

pub struct MetricsCollector {
    pub registry: Registry,

    // Player Connection Metrics
    pub player_connections_total: CounterVec,
    pub player_disconnections_total: CounterVec,
    pub active_player_sessions: GaugeVec,
    pub player_connection_duration: HistogramVec,

    // Proxy Metrics
    pub proxy_messages_sent: CounterVec,
    pub proxy_messages_received: CounterVec,
    pub proxy_message_size_bytes: HistogramVec,
    pub proxy_message_latency: HistogramVec,
    pub proxy_errors_total: CounterVec,

    // Session Metrics
    pub session_created_total: CounterVec,
    pub session_timeout_total: CounterVec,
    pub session_reused_total: CounterVec,
    pub active_sessions: GaugeVec,

    // rcssserver Process Metrics
    pub rcssserver_status: GaugeVec,
    pub rcssserver_restarts_total: Counter,
    pub rcssserver_uptime_seconds: Gauge,
    pub game_timestep: Gauge,
    pub game_status: GaugeVec,

    // HTTP Server Metrics
    pub http_requests_total: CounterVec,
    pub http_request_duration: HistogramVec,
    pub websocket_upgrades_total: Counter,
    pub websocket_upgrade_failures: Counter,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let registry = Registry::new();

        // Player Connection Metrics
        let player_connections_total = CounterVec::new(
            Opts::new(
                "rcss_player_connections_total",
                "Total number of player connections by protocol"
            ),
            &["protocol"]
        ).unwrap();

        let player_disconnections_total = CounterVec::new(
            Opts::new(
                "rcss_player_disconnections_total",
                "Total number of player disconnections by protocol and reason"
            ),
            &["protocol", "reason"]
        ).unwrap();

        let active_player_sessions = GaugeVec::new(
            Opts::new(
                "rcss_active_player_sessions",
                "Current number of active player sessions by protocol"
            ),
            &["protocol"]
        ).unwrap();

        let player_connection_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "rcss_player_connection_duration_seconds",
                "Duration of player connections in seconds"
            )
            .buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0, 1800.0, 3600.0]),
            &["protocol"]
        ).unwrap();

        // Proxy Metrics
        let proxy_messages_sent = CounterVec::new(
            Opts::new(
                "rcss_proxy_messages_sent_total",
                "Total number of messages sent through proxy by protocol"
            ),
            &["protocol", "direction"]
        ).unwrap();

        let proxy_messages_received = CounterVec::new(
            Opts::new(
                "rcss_proxy_messages_received_total",
                "Total number of messages received through proxy by protocol"
            ),
            &["protocol", "direction"]
        ).unwrap();

        let proxy_message_size_bytes = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "rcss_proxy_message_size_bytes",
                "Size of messages passing through proxy in bytes"
            )
            .buckets(vec![64.0, 256.0, 512.0, 1024.0, 2048.0, 4096.0, 8192.0]),
            &["protocol", "direction"]
        ).unwrap();

        let proxy_message_latency = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "rcss_proxy_message_latency_seconds",
                "Latency of message processing in proxy"
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["protocol"]
        ).unwrap();

        let proxy_errors_total = CounterVec::new(
            Opts::new(
                "rcss_proxy_errors_total",
                "Total number of proxy errors by protocol and type"
            ),
            &["protocol", "error_type"]
        ).unwrap();

        // Session Metrics
        let session_created_total = CounterVec::new(
            Opts::new(
                "rcss_session_created_total",
                "Total number of sessions created by protocol"
            ),
            &["protocol"]
        ).unwrap();

        let session_timeout_total = CounterVec::new(
            Opts::new(
                "rcss_session_timeout_total",
                "Total number of session timeouts by protocol"
            ),
            &["protocol"]
        ).unwrap();

        let session_reused_total = CounterVec::new(
            Opts::new(
                "rcss_session_reused_total",
                "Total number of session reuses by protocol"
            ),
            &["protocol"]
        ).unwrap();

        let active_sessions = GaugeVec::new(
            Opts::new(
                "rcss_active_sessions",
                "Current number of active sessions by protocol"
            ),
            &["protocol"]
        ).unwrap();

        // rcssserver Process Metrics
        let rcssserver_status = GaugeVec::new(
            Opts::new(
                "rcss_server_status",
                "Status of rcssserver process (0=dead, 1=init, 2=booting, 3=running, 4=returned)"
            ),
            &["status"]
        ).unwrap();

        let rcssserver_restarts_total = Counter::new(
            "rcss_server_restarts_total",
            "Total number of rcssserver restarts"
        ).unwrap();

        let rcssserver_uptime_seconds = Gauge::new(
            "rcss_server_uptime_seconds",
            "Uptime of rcssserver process in seconds"
        ).unwrap();

        let game_timestep = Gauge::new(
            "rcss_game_timestep",
            "Current game timestep"
        ).unwrap();

        let game_status = GaugeVec::new(
            Opts::new(
                "rcss_game_status",
                "Game status (0=uninitialized, 1=idle, 2=simulating, 3=finished, 4=shutdown)"
            ),
            &["status"]
        ).unwrap();

        // HTTP Server Metrics
        let http_requests_total = CounterVec::new(
            Opts::new(
                "rcss_http_requests_total",
                "Total number of HTTP requests by method and path"
            ),
            &["method", "path", "status"]
        ).unwrap();

        let http_request_duration = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "rcss_http_request_duration_seconds",
                "Duration of HTTP requests in seconds"
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]),
            &["method", "path"]
        ).unwrap();

        let websocket_upgrades_total = Counter::new(
            "rcss_websocket_upgrades_total",
            "Total number of successful WebSocket upgrades"
        ).unwrap();

        let websocket_upgrade_failures = Counter::new(
            "rcss_websocket_upgrade_failures_total",
            "Total number of failed WebSocket upgrades"
        ).unwrap();

        // Register all metrics
        registry.register(Box::new(player_connections_total.clone())).unwrap();
        registry.register(Box::new(player_disconnections_total.clone())).unwrap();
        registry.register(Box::new(active_player_sessions.clone())).unwrap();
        registry.register(Box::new(player_connection_duration.clone())).unwrap();

        registry.register(Box::new(proxy_messages_sent.clone())).unwrap();
        registry.register(Box::new(proxy_messages_received.clone())).unwrap();
        registry.register(Box::new(proxy_message_size_bytes.clone())).unwrap();
        registry.register(Box::new(proxy_message_latency.clone())).unwrap();
        registry.register(Box::new(proxy_errors_total.clone())).unwrap();

        registry.register(Box::new(session_created_total.clone())).unwrap();
        registry.register(Box::new(session_timeout_total.clone())).unwrap();
        registry.register(Box::new(session_reused_total.clone())).unwrap();
        registry.register(Box::new(active_sessions.clone())).unwrap();

        registry.register(Box::new(rcssserver_status.clone())).unwrap();
        registry.register(Box::new(rcssserver_restarts_total.clone())).unwrap();
        registry.register(Box::new(rcssserver_uptime_seconds.clone())).unwrap();
        registry.register(Box::new(game_timestep.clone())).unwrap();
        registry.register(Box::new(game_status.clone())).unwrap();

        registry.register(Box::new(http_requests_total.clone())).unwrap();
        registry.register(Box::new(http_request_duration.clone())).unwrap();
        registry.register(Box::new(websocket_upgrades_total.clone())).unwrap();
        registry.register(Box::new(websocket_upgrade_failures.clone())).unwrap();

        Self {
            registry,
            player_connections_total,
            player_disconnections_total,
            active_player_sessions,
            player_connection_duration,
            proxy_messages_sent,
            proxy_messages_received,
            proxy_message_size_bytes,
            proxy_message_latency,
            proxy_errors_total,
            session_created_total,
            session_timeout_total,
            session_reused_total,
            active_sessions,
            rcssserver_status,
            rcssserver_restarts_total,
            rcssserver_uptime_seconds,
            game_timestep,
            game_status,
            http_requests_total,
            http_request_duration,
            websocket_upgrades_total,
            websocket_upgrade_failures,
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
