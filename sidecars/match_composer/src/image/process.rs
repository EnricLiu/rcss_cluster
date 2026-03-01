use std::path::Path;
use std::process::Stdio;

use tokio::io::AsyncWriteExt;
use tokio::task::JoinHandle;
use tokio::sync::watch;
use tokio::process::Command;

use common::process::{Process, ProcessStatus};
use log::{info, warn};

#[derive(Debug)]
pub struct ImageProcess {
    pub process: Process,
    pub log_task: Option<JoinHandle<Result<()>>>,
}

impl ImageProcess {
    pub(crate) fn spawn(mut command: Command, log: Option<Box<Path>>) -> Result<Self> {
        let command = command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let child = command.spawn().map_err(Error::ChildFailedSpawn)?;
        let process = Process::new(child)?;

        let log_task = if let Some(log_path) = log {
            let log_task = Self::spawn_log_task(&process, &log_path)?;
            Some(log_task)
        } else { None };
        
        Ok(Self {
            process,
            log_task,
        })
    }
    
    pub(crate) fn status_watch(&self) -> watch::Receiver<ProcessStatus> {
        self.process.status_watch()
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        if let Err(e) = self.process.shutdown().await {
            warn!("Error shutting down bot process: {e}");
        }
        if let Some(log_task) = self.log_task.take() {
            log_task.abort();
        }
        Ok(())
    }

    fn spawn_log_task(process: &Process, log_path: &Path) -> Result<JoinHandle<Result<()>>> {
        std::fs::create_dir_all(log_path.parent().unwrap_or_else(|| Path::new("/")))
            .map_err(Error::BotLogOpen)?;

        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(Error::BotLogOpen)?;

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
            let mut file = tokio::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_path)
                .await.map_err(Error::BotLogOpen)?;

            let mut buf = Vec::with_capacity(64);
            let _ = loop {
                tokio::select! {
                _ = &mut process_end => {
                    info!("Bot process ended, stopping log task");
                    break Ok(());
                },

                res = stdout.recv() => {
                    match res {
                        Ok(out) => {
                            buf.push(out);
                            if buf.len() >= 64 {
                                let data = buf.join("\n") + "\n";
                                if let Err(e) = file.write_all(data.as_bytes()).await {
                                    warn!("Error writing stdout to bot log file: {e}");
                                    break Err(Error::BotLogWrite(e))
                                }
                                buf.clear();
                            }
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
                            buf.push(err);
                            if buf.len() >= 64 {
                                let data = buf.join("\n") + "\n";
                                if let Err(e) = file.write_all(data.as_bytes()).await {
                                    warn!("Error writing stderr to bot log file: {e}");
                                    break Err(Error::BotLogWrite(e))
                                }
                                buf.clear();
                            }
                        },
                        Err(e) => {
                            info!("Error reading bot stderr: {e}");
                            break Ok(());
                        }
                    }
                },
            }
            }?;

            if !buf.is_empty() {
                let data = buf.join("\n") + "\n";
                file.write_all(data.as_bytes()).await.map_err(|e| {
                    warn!("Error writing stdout to bot log file: {e}");
                    Error::BotLogWrite(e)
                })?
            };

            Ok(())
        });

        Ok(task)
    }
}


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Process(#[from] common::process::ProcessError),
    
    #[error("Failed to spawn child process, {0}")]
    ChildFailedSpawn(#[source] std::io::Error),

    #[error("Failed to open bot log file, {0}")]
    BotLogOpen(#[source] std::io::Error),

    #[error("Failed to write bot log file, {0}")]
    BotLogWrite(#[source] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
