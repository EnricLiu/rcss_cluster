use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use log::{error, info, warn};
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{mpsc, watch, OnceCell, SetError};
use tokio::task::JoinHandle;
use common::process::{Process, ProcessStatus};
use crate::declarations::Unum;
use crate::model::player::PlayerBaseModel;
use crate::policy::Policy;

const STDOUT_LOG_PREFIX: &[u8] = b"[stdout] ";
const STDERR_LOG_PREFIX: &[u8] = b"[stderr] ";
const LOG_NEWLINE: &[u8] = b"\n";
const LOG_FLUSH_THRESHOLD: usize = 16; // 16 lines
const LOG_FLUSH_INTERVAL: Duration = Duration::from_secs(15);


#[async_trait::async_trait]
pub trait Player: Debug + Send + Sync + 'static {
    fn model(&self) -> &PlayerBaseModel;
    fn status_watch(&self) -> Option<watch::Receiver<ProcessStatus>>;
    async fn spawn(&self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
}


#[derive(Debug)]
pub struct PolicyPlayer<Config: Policy + Sync + Send + 'static> {
    pub config: Config,
    pub process: OnceCell<Process>,
    pub logging_task: OnceCell<JoinHandle<Result<()>>>,
}

impl<Config: Policy + Sync + Send + 'static> PolicyPlayer<Config> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            process: OnceCell::new(),
            logging_task: OnceCell::new(),
        }
    }

    async fn create_log_path(&self) -> Result<()> {
        if let Some(dir) = self.config.log_dir() {
            tokio::fs::create_dir_all(&dir).await.map_err(|e| Error::LogMkdir(dir, e))?;
        }
        Ok(())
    }

    /// log_path should have existed parents and should be a file path (not a directory)
    fn spawn_log_task(process: &Process, log_path: impl AsRef<Path>) -> Result<JoinHandle<Result<()>>> {
        let log_path = log_path.as_ref();
        if log_path.extension().is_none() { return Err(Error::FileLogInvalidPath(log_path.to_path_buf())) }

        let mut process_end = {
            let mut status = process.status_watch();
            let process_end = async move {
                match status.wait_for(|s| s.is_finished()).await {
                    Ok(_) => info!("Bot process finished, stopping log task"),
                    Err(e) => warn!("Bot process status watch error: {e}, stopping log task"),
                }
            };
            Box::pin(process_end)
        };

        let mut stdout = process.subscribe_stdout();
        let mut stderr = process.subscribe_stderr();
        let log_path = log_path.to_path_buf();

        let task = tokio::spawn(async move {
            let file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .await.map_err(Error::FileLogOpen)?;

            let mut file = BufWriter::new(file);

            #[inline]
            async fn write_log_flush(
                file: &mut BufWriter<File>,
                prefix: &[u8], data: &str,
                idx: &mut usize, thresh: usize
            ) -> Result<()> {
                file.write_all(prefix).await.map_err(Error::FileLogWrite)?;
                file.write_all(data.as_bytes()).await.map_err(Error::FileLogWrite)?;
                file.write_all(LOG_NEWLINE).await.map_err(Error::FileLogWrite)?;
                *idx += 1;
                if *idx >= thresh {
                    file.flush().await.map_err(Error::FileLogWrite)?;
                    *idx = 0;
                }
                Ok(())
            }

            let mut line_cnt = 0;
            let mut flush_interval = tokio::time::interval(LOG_FLUSH_INTERVAL);

            let _ = loop { tokio::select! {
                _ = &mut process_end => {
                    break <Result<()>>::Ok(());
                },

                res = stdout.recv() => {
                    match res {
                        Ok(out) => {
                            write_log_flush(
                                &mut file,
                                STDOUT_LOG_PREFIX, &out,
                                &mut line_cnt, LOG_FLUSH_THRESHOLD
                            ).await?;
                        },
                        Err(e) => {
                            info!("Error reading bot stdout: {e}");
                            break Ok(());
                        }
                    }
                },
                res = stderr.recv() => {
                    match res {
                        Ok(err) => {
                            write_log_flush(
                                &mut file,
                                STDERR_LOG_PREFIX, &err,
                                &mut line_cnt, LOG_FLUSH_THRESHOLD
                            ).await?;
                        },
                        Err(e) => {
                            info!("Error reading bot stderr: {e}");
                            break Ok(());
                        }
                    }
                },

                _ = flush_interval.tick() => {
                    file.flush().await.map_err(Error::FileLogWrite)?;
                    line_cnt = 0;
                },
            }}?;

            file.flush().await.map_err(Error::FileLogWrite)?;

            Ok(())
        });

        Ok(task)
    }
}

#[async_trait::async_trait]
impl<Config: Policy + Sync + Send + 'static> Player for PolicyPlayer<Config> {
    fn model(&self) -> &PlayerBaseModel {
        self.config.info()
    }

    fn status_watch(&self) -> Option<watch::Receiver<ProcessStatus>> {
        self.process.get().map(|p| p.status_watch())
    }

    async fn spawn(&self) -> Result<()> {
        self.create_log_path().await?;

        let mut command = self.config.command();
        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let process = self.process.get_or_try_init(|| async {
            let child = command.spawn().map_err(Error::ChildFailedSpawn)?;
            <Result<_>>::Ok(Process::new(child)?)
        }).await?;
        
        if let Some(log_root) = &self.config.log_dir() {
            let info = self.model();
            let path = log_root.join(
                format!("{}-{}-stdio.log", info.team, info.unum)
            );

            let logging_task = Self::spawn_log_task(&process, path)?;
            if let Err(e) = self.logging_task.set(logging_task) {
                set_task_error_abort("logging", e);
            }
        }

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        if !self.process.initialized() {
            return Err(Error::NotRunning)
        }

        self.process.get_mut().unwrap().shutdown().await?;
        if let Some(task) = self.logging_task.get() {
            task.abort();
        }

        Ok(())
    }
}

fn set_task_error_abort<T>(task_name: &str, err: SetError<JoinHandle<T>>) {
    let task = match err {
        SetError::InitializingError(task) => {
            error!("Player {task_name} task initialization error");
            task
        },
        SetError::AlreadyInitializedError(task) => {
            warn!("Player {task_name} task already initialized, skipping");
            task
        },
    };
    task.abort()
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Process(#[from] common::process::ProcessError),

    #[error("Failed to spawn child process, {0}")]
    ChildFailedSpawn(#[source] std::io::Error),

    #[error("Player is not running")]
    NotRunning,

    #[error("Player is already running")]
    IsRunning,

    #[error("Failed to create bot log directory, {0}")]
    LogMkdir(PathBuf, #[source] std::io::Error),

    #[error("Failed to open bot log file, {0}")]
    FileLogOpen(#[source] std::io::Error),

    #[error("Failed to write bot log file, {0}")]
    FileLogWrite(#[source] std::io::Error),

    #[error("Invalid bot log file path: {0}, should be a file path")]
    FileLogInvalidPath(PathBuf),
}

pub type Result<T> = std::result::Result<T, Error>;

