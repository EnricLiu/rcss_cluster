mod client;
mod config;
mod error;
mod manifest;

use chrono::Utc;
use clap::Parser;
use client::{Allocation, AllocatorClient, GameServerClient, ServerObservation};
use config::{MatchConfigInput, build_allocate_request};
use error::{Error, Result};
use log::{debug, error, info, warn};
use manifest::{
    CleanupReport, MatchManifest, MatchOutcome, SummaryRecord, append_summary, write_manifest,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(
    name = "job-traj-pred-datagen",
    about = "Generate trajectory-prediction datasets from allocated rcss_cluster matches"
)]
struct Args {
    #[arg(
        long,
        env = "ALLOCATOR_URL",
        default_value = "http://rcss-env-allocator.rcss-gateway-dev.svc.cluster.local/gs/allocate"
    )]
    allocator_url: String,

    #[arg(
        long,
        env = "OUTPUT_ROOT",
        default_value = "/mnt/output/datasets/traj-pred"
    )]
    output_root: PathBuf,

    #[arg(long, env = "DATAGEN_RUN_ID")]
    run_id: Option<String>,

    #[arg(long, env = "JOB_COMPLETION_INDEX")]
    completion_index: Option<u32>,

    #[arg(long, env = "MATCHES_PER_INDEX", default_value_t = 1)]
    matches_per_index: u32,

    #[arg(long, env = "MATCH_INDEX_OFFSET", default_value_t = 0)]
    match_index_offset: u64,

    #[arg(long, env = "MATCH_TIME_UP", default_value_t = 6000)]
    match_time_up: u16,

    #[arg(long, env = "POLL_INTERVAL_MS", default_value_t = 2000)]
    poll_interval_ms: u64,

    #[arg(long, env = "MATCH_TIMEOUT_S", default_value_t = 900)]
    match_timeout_s: u64,

    #[arg(long, env = "HEARTBEAT_INTERVAL_S", default_value_t = 60)]
    heartbeat_interval_s: u64,

    #[arg(long, env = "HTTP_TIMEOUT_S", default_value_t = 20)]
    http_timeout_s: u64,

    #[arg(long, env = "PREFER_POD_IP", default_value_t = true)]
    prefer_pod_ip: bool,

    #[arg(long, env = "POD_HTTP_PORT", default_value_t = 6666)]
    pod_http_port: u16,
}

#[derive(Debug, Clone)]
struct RunSettings {
    run_id: String,
    job_index: u32,
    output_root: PathBuf,
    matches_per_index: u32,
    match_index_offset: u64,
    match_time_up: u16,
    poll_interval: Duration,
    match_timeout: Duration,
    heartbeat_interval: Duration,
    prefer_pod_ip: bool,
    pod_http_port: u16,
}

impl RunSettings {
    fn from_args(args: Args) -> Self {
        let run_id = args
            .run_id
            .unwrap_or_else(|| format!("run-{}", Uuid::now_v7()));
        Self {
            run_id,
            job_index: args.completion_index.unwrap_or(0),
            output_root: args.output_root,
            matches_per_index: args.matches_per_index,
            match_index_offset: args.match_index_offset,
            match_time_up: args.match_time_up,
            poll_interval: Duration::from_millis(args.poll_interval_ms),
            match_timeout: Duration::from_secs(args.match_timeout_s),
            heartbeat_interval: Duration::from_secs(args.heartbeat_interval_s),
            prefer_pod_ip: args.prefer_pod_ip,
            pod_http_port: args.pod_http_port,
        }
    }

    fn global_match_index(&self, local_match_index: u32) -> u64 {
        self.match_index_offset
            + u64::from(self.job_index) * u64::from(self.matches_per_index)
            + u64::from(local_match_index)
    }

    fn match_id(&self, global_match_index: u64) -> String {
        format!("idx{:05}-match{:08}", self.job_index, global_match_index)
    }

    fn match_output_dir(&self, match_id: &str) -> PathBuf {
        self.output_root
            .join(&self.run_id)
            .join("matches")
            .join(match_id)
    }

    fn summary_path(&self) -> PathBuf {
        self.output_root
            .join(&self.run_id)
            .join(format!("index-{:05}", self.job_index))
            .join("summary.jsonl")
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    if let Err(err) = run(args).await {
        error!("{err}");
        std::process::exit(1);
    }
}

async fn run(args: Args) -> Result<()> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(args.http_timeout_s))
        .build()
        .map_err(|source| Error::http("build_http_client", source))?;

    let allocator = AllocatorClient::new(http.clone(), &args.allocator_url);
    let settings = RunSettings::from_args(args);

    info!(
        "starting datagen run_id={} job_index={} matches_per_index={}",
        settings.run_id, settings.job_index, settings.matches_per_index
    );

    let mut failures = 0u32;
    for local_match_index in 0..settings.matches_per_index {
        let manifest = run_one_match(
            &settings,
            allocator.clone(),
            http.clone(),
            local_match_index,
        )
        .await?;
        if manifest.outcome != MatchOutcome::Succeeded {
            failures += 1;
        }
    }

    if failures > 0 {
        Err(Error::MatchFailures)
    } else {
        Ok(())
    }
}

async fn run_one_match(
    settings: &RunSettings,
    allocator: AllocatorClient,
    http: reqwest::Client,
    local_match_index: u32,
) -> Result<MatchManifest> {
    let global_match_index = settings.global_match_index(local_match_index);
    let match_id = settings.match_id(global_match_index);

    let generated = build_allocate_request(&MatchConfigInput {
        global_match_index,
        time_up: settings.match_time_up,
    });

    let mut manifest = MatchManifest::new(
        &settings.run_id,
        settings.job_index,
        &match_id,
        global_match_index,
        generated.side_order,
        Utc::now(),
        settings.match_output_dir(&match_id),
    );

    info!(
        "starting match_id={} side_order={} global_index={}",
        match_id, manifest.side_order, global_match_index
    );

    let result = execute_match(
        settings,
        &allocator,
        http,
        &generated.request,
        &mut manifest,
    )
    .await;
    match result {
        Ok(()) => manifest.finish(MatchOutcome::Succeeded, None),
        Err(err) => {
            warn!("match_id={} failed: {err}", match_id);
            manifest.finish(MatchOutcome::Failed, Some(err.to_string()));
        }
    }

    let manifest_path = write_manifest(&manifest)?;
    append_summary(
        &settings.summary_path(),
        &SummaryRecord {
            run_id: &manifest.run_id,
            job_index: manifest.job_index,
            match_id: &manifest.match_id,
            global_match_index: manifest.global_match_index,
            outcome: manifest.outcome,
            manifest_path: &manifest_path,
            gameserver_name: manifest.allocation.as_ref().map(|a| a.name.as_str()),
            error: manifest.error.as_deref(),
        },
    )?;

    Ok(manifest)
}

async fn execute_match(
    settings: &RunSettings,
    allocator: &AllocatorClient,
    http: reqwest::Client,
    request: &serde_json::Value,
    manifest: &mut MatchManifest,
) -> Result<()> {
    let allocation = allocator.allocate(request).await?;
    manifest.allocation = Some(allocation.clone());

    let server = GameServerClient::from_allocation(
        http,
        &allocation,
        settings.prefer_pod_ip,
        settings.pod_http_port,
    )?;
    manifest.primary_server_url = server.primary_base_url().map(ToString::to_string);

    let heartbeat_stop = Arc::new(AtomicBool::new(false));
    let heartbeat_task = spawn_heartbeat(
        allocator.clone(),
        allocation.name.clone(),
        settings.heartbeat_interval,
        heartbeat_stop.clone(),
    );

    let flow_result = execute_match_with_server(settings, &server, manifest).await;
    let cleanup = cleanup_match(
        allocator,
        &server,
        &allocation,
        heartbeat_stop,
        heartbeat_task,
    )
    .await;
    manifest.cleanup = cleanup;

    flow_result
}

async fn execute_match_with_server(
    settings: &RunSettings,
    server: &GameServerClient,
    manifest: &mut MatchManifest,
) -> Result<()> {
    let first = wait_for_observation(
        settings,
        server,
        &manifest.match_id,
        "become reachable",
        |_| true,
    )
    .await?;
    manifest.final_observation = Some(first.clone());

    if !first.is_started() {
        let ready = wait_for_observation(
            settings,
            server,
            &manifest.match_id,
            "report match-composer readiness",
            |obs| obs.is_started() || obs.match_composer_ready(),
        )
        .await?;
        manifest.final_observation = Some(ready.clone());

        if !ready.is_started() {
            server.trainer_start().await?;
            manifest.trainer_start_sent = true;
        } else {
            debug!(
                "match_id={} already started before trainer start call",
                manifest.match_id
            );
        }
    }

    let finished = wait_for_observation(settings, server, &manifest.match_id, "finish", |obs| {
        let is_finished = obs.is_finished(settings.match_time_up);

        if !is_finished {
            info!(
                "[RUNNING] match_id={}: ts={}, uptime={:?}ms",
                manifest.match_id,
                obs.service.timestep.unwrap_or(0),
                obs.service.uptime_ms.unwrap_or(0),
            )
        } else {
            info!(
                "[FINISHED] match_id={}: ts={}, uptime={:?}ms",
                manifest.match_id,
                obs.service.timestep.unwrap_or(0),
                obs.service.uptime_ms.unwrap_or(0),
            );
        }

        is_finished
    })
    .await?;
    manifest.final_observation = Some(finished);

    Ok(())
}

async fn wait_for_observation<F>(
    settings: &RunSettings,
    server: &GameServerClient,
    match_id: &str,
    phase: &'static str,
    predicate: F,
) -> Result<ServerObservation>
where
    F: Fn(&ServerObservation) -> bool,
{
    let deadline = Instant::now() + settings.match_timeout;
    let mut last_error = None;

    loop {
        match server.status().await {
            Ok(observation) => {
                if predicate(&observation) {
                    return Ok(observation);
                }
            }
            Err(err) => {
                debug!("match_id={match_id} status poll during {phase} failed: {err}");
                last_error = Some(err.to_string());
            }
        }

        if Instant::now() >= deadline {
            if let Some(err) = last_error {
                warn!("match_id={match_id} last status error before timeout: {err}");
            }
            return Err(Error::Timeout {
                match_id: match_id.to_string(),
                phase,
            });
        }

        client::sleep_interval(settings.poll_interval).await;
    }
}

fn spawn_heartbeat(
    allocator: AllocatorClient,
    gs_name: String,
    interval: Duration,
    stop: Arc<AtomicBool>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while !stop.load(Ordering::Relaxed) {
            tokio::time::sleep(interval).await;
            if stop.load(Ordering::Relaxed) {
                break;
            }
            if let Err(err) = allocator.heartbeat(&gs_name).await {
                warn!("heartbeat for GameServer[{gs_name}] failed: {err}");
            }
        }
    })
}

async fn cleanup_match(
    allocator: &AllocatorClient,
    server: &GameServerClient,
    allocation: &Allocation,
    heartbeat_stop: Arc<AtomicBool>,
    heartbeat_task: JoinHandle<()>,
) -> CleanupReport {
    heartbeat_stop.store(true, Ordering::Relaxed);
    let _ = heartbeat_task.await;

    let server_shutdown = match server.shutdown().await {
        Ok(_) => Some("ok".to_string()),
        Err(err) => {
            warn!(
                "server shutdown for GameServer[{}] failed: {err}",
                allocation.name
            );
            Some(format!("error: {err}"))
        }
    };

    let allocator_drop = match allocator.drop_direct_gs(&allocation.name).await {
        Ok(_) => Some("ok".to_string()),
        Err(err) => {
            warn!("drop GameServer[{}] failed: {err}", allocation.name);
            Some(format!("error: {err}"))
        }
    };

    CleanupReport {
        server_shutdown,
        allocator_drop,
    }
}
