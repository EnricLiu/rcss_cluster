use std::io;
use std::process::ExitStatus;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::time::Duration;

use log::{debug, error, info, warn};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use super::*;
use super::builder::ServerProcessSpawner;

pub struct ServerProcess {
    pid:        Arc<AtomicU32>,
    handle:     JoinHandle<(io::Result<ExitStatus>, Child)>,
    sig_tx:     mpsc::Sender<Signal>,
    stdout_rx:  mpsc::Receiver<String>,
    stderr_rx:  mpsc::Receiver<String>,
}

impl ServerProcess {
    pub const TERM_TIMEOUT_S: Duration = Duration::from_secs(5);

    pub async fn spawner(pgm_name: &'static str) -> ServerProcessSpawner {
        ServerProcessSpawner::new(pgm_name).await
    }

    pub(crate) async fn try_from(child: Child) -> Result<Self> {
        let pid = child.id().ok_or(Error::ChildAlreadyCompleted)?;
        let arc_pid = Arc::new(AtomicU32::new(pid));
        let pid = Pid::from_raw(pid as i32);

        let (sig_tx, mut sig_rx) = mpsc::channel(4);
        let (stdout_tx, stdout_rx) = mpsc::channel(16);
        let (stderr_tx, stderr_rx) = mpsc::channel(16);

        let arc_pid_ = Arc::clone(&arc_pid);
        let handle = tokio::spawn(async move {
            let mut child = child;
            let arc_pid = arc_pid_;
            let stdout = child.stdout.take().expect("Failed to open stdout");
            let stderr = child.stderr.take().expect("Failed to open stderr");

            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            loop {
                tokio::select! {
                    status = child.wait() => {
                        info!("RcssServer child process exited with status: {:?}", status);
                        arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
                        return (status, child);
                    }

                    Some(sig) = sig_rx.recv() => {
                        match kill(pid, sig) {
                            Ok(_) => info!("Sent signal {:?} to child process", sig),
                            Err(e) => {
                                error!("Failed to send signal {:?} to child process: {}", sig, e);
                            }
                        }
                    }

                    result = stdout_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => {
                                if stdout_tx.send(line).await.is_err() {
                                    // Channel closed, probably the receiver was dropped.
                                    break;
                                }
                            }
                            Ok(None) => break, // stdout closed
                            Err(e) => {
                                error!("Error reading from stdout: {}", e);
                                break;
                            }
                        }
                    }

                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => {
                                if stderr_tx.send(line).await.is_err() {
                                    break;
                                }
                            }
                            Ok(None) => break, // stderr closed
                            Err(e) => {
                                error!("Error reading from stderr: {}", e);
                                break;
                            }
                        }
                    }
                }
            }

            let status = child.wait().await;
            arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
            (status, child)
        });

        Ok(Self {
            handle,
            pid: arc_pid,
            sig_tx,
            stdout_rx,
            stderr_rx,
        })
    }

    pub async fn terminate(self) -> Result<ExitStatus> {
        let signal = Signal::SIGTERM;

        self.sig_tx.send(signal).await.map_err(Error::SignalSend)?;
        let (status, child) = tokio::time::timeout(Self::TERM_TIMEOUT_S, self.handle).await
            .map_err(Error::ProcessJoinTimeout)?
            .map_err(Error::ProcessJoin)?;

        let status = match status {
            Ok(status) => {
                if status.success() {
                    debug!("RcssServer::terminate: process exited successfully");
                } else {
                    warn!("RcssServer::terminate: process exited with status: {status:?}");
                }
                status
            },
            Err(e) => {
                warn!("RcssServer::terminate: wait on process exits with error, trying KILL...");

                let mut child = child;
                let pid = child.id();

                if  let Some(pid) = pid &&
                    let Ok(_) = kill(Pid::from_raw(pid as i32), Signal::SIGKILL) &&
                    let Ok(status) = child.wait().await // todo!("timeout")
                {
                    warn!("RcssServer::terminate: process KILLed successfully with pid: {}", pid);
                    status
                } else {
                    return Err(Error::FatalProcessWindingUp {
                        pid,
                        signal,
                        error: e,
                    })
                }
            }
        };

        Ok(status)
    }

    pub fn pid(&self) -> Option<u32> {
        let pid = self.pid.load(std::sync::atomic::Ordering::SeqCst);
        (pid != 0).then_some(pid)
    }
}
