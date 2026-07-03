use crate::client::{Allocation, ServerObservation};
use crate::config::SideOrder;
use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct MatchManifest {
    pub run_id: String,
    pub job_index: u32,
    pub match_id: String,
    pub global_match_index: u64,
    pub side_order: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub output_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_server_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocation: Option<Allocation>,
    pub trainer_start_sent: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_observation: Option<ServerObservation>,
    pub cleanup: CleanupReport,
    pub outcome: MatchOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CleanupReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_shutdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allocator_drop: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MatchOutcome {
    Succeeded,
    Failed,
}

impl MatchManifest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        run_id: &str,
        job_index: u32,
        match_id: &str,
        global_match_index: u64,
        side_order: SideOrder,
        started_at: DateTime<Utc>,
        output_dir: PathBuf,
    ) -> Self {
        Self {
            run_id: run_id.to_string(),
            job_index,
            match_id: match_id.to_string(),
            global_match_index,
            side_order: side_order.as_str().to_string(),
            started_at,
            finished_at: started_at,
            output_dir,
            primary_server_url: None,
            allocation: None,
            trainer_start_sent: false,
            final_observation: None,
            cleanup: CleanupReport::default(),
            outcome: MatchOutcome::Failed,
            error: None,
        }
    }

    pub fn finish(&mut self, outcome: MatchOutcome, error: Option<String>) {
        self.finished_at = Utc::now();
        self.outcome = outcome;
        self.error = error;
    }
}

#[derive(Debug, Serialize)]
pub struct SummaryRecord<'a> {
    pub run_id: &'a str,
    pub job_index: u32,
    pub match_id: &'a str,
    pub global_match_index: u64,
    pub outcome: MatchOutcome,
    pub manifest_path: &'a Path,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gameserver_name: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'a str>,
}

pub fn write_manifest(manifest: &MatchManifest) -> Result<PathBuf> {
    create_dir_all(&manifest.output_dir)
        .map_err(|source| Error::io(&manifest.output_dir, source))?;

    let path = manifest.output_dir.join("manifest.json");
    let content = serde_json::to_vec_pretty(manifest)
        .map_err(|source| Error::decode("serialize_manifest", source))?;
    std::fs::write(&path, content).map_err(|source| Error::io(&path, source))?;
    Ok(path)
}

pub fn append_summary(path: &Path, record: &SummaryRecord<'_>) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| Error::io(path, source))?;
    serde_json::to_writer(&mut file, record)
        .map_err(|source| Error::decode("serialize_summary", source))?;
    file.write_all(b"\n")
        .map_err(|source| Error::io(path, source))?;
    Ok(())
}
