use std::collections::VecDeque;
use std::io;
use std::process::ExitStatus;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicU32;
use std::time::Duration;
use log::{debug, error, info, trace, warn};
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::{mpsc, watch, RwLock};
use tokio::task::JoinHandle;

use common::ringbuf::OverwriteRB;

use super::builder::ServerProcessSpawner;
use super::*;

pub const READY_LINE: &str = "Hit CTRL-C to exit";
pub const STDIO_REPORT_DURATION: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct ServerProcess {
    pid: Arc<AtomicU32>,
    handle: JoinHandle<(io::Result<ExitStatus>, Child)>,
    sig_tx: mpsc::Sender<Signal>,

    // stdio_report_duration: Duration,
    stdout: Arc<RwLock<OverwriteRB<String, 32>>>,
    stderr: Arc<RwLock<OverwriteRB<String, 32>>>,

    status_rx: watch::Receiver<Status>,
}

impl ServerProcess {
    pub const TERM_TIMEOUT_S: Duration = Duration::from_secs(5);

    pub async fn spawner(pgm_name: &'static str) -> ServerProcessSpawner {
        ServerProcessSpawner::new(pgm_name).await
    }

    pub(crate) async fn try_from(mut child: Child) -> Result<Self> {
        match child.try_wait() {
            Ok(None) => {},
            Ok(Some(status)) => return Err(Error::ChildAlreadyCompleted(status)),
            Err(e) => return Err(Error::ChildUntrackableWithoutPid(e)), // todo!("handle it with `child` internally")
        }

        let pid = child.id().ok_or(Error::ChildRunningWithoutPid)?;

        let arc_pid = Arc::new(AtomicU32::new(pid));
        let pid = Pid::from_raw(pid as i32);

        let (status_tx, status_rx) = watch::channel(Status::Init);
        let (sig_tx, mut sig_rx) = mpsc::channel(4);
        let stdout_rb = Arc::new(RwLock::new(OverwriteRB::new()));
        let stderr_rb = Arc::new(RwLock::new(OverwriteRB::new()));

        let arc_pid_ = Arc::clone(&arc_pid);
        let stdout_rb_ = Arc::clone(&stdout_rb);
        let stderr_rb_ = Arc::clone(&stderr_rb);
        let handle = tokio::spawn(async move {
            let mut child = child;
            let arc_pid = arc_pid_;

            let mut stdout_reader = {
                let stdout = child.stdout.take().ok_or_else(|| {
                    error!("Failed to capture stdout from child process");
                    io::Error::new(io::ErrorKind::Other, "stdout not available")
                }).expect("stdout should be available with Stdio::piped()");

                BufReader::new(stdout).lines()
            };

            let mut stderr_reader = {
                let stderr = child.stderr.take().ok_or_else(|| {
                    error!("Failed to capture stderr from child process");
                    io::Error::new(io::ErrorKind::Other, "stderr not available")
                }).expect("stderr should be available with Stdio::piped()");

                BufReader::new(stderr).lines()
            };

            if status_tx.send(Status::Booting).is_err() {
                warn!("Failed to send Booting status: receiver dropped");
            }

            let mut stdout_buf = Vec::with_capacity(32);
            let mut stderr_buf = Vec::with_capacity(8);

            loop {
                tokio::select! {
                    status = child.wait() => {
                        info!("RcssServer child process exited with status: {:?}", status);
                        arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
                        let status_send = match &status {
                            Ok(status) => Status::Returned(*status),
                            Err(e) => Status::Dead(e.to_string()),
                        };
                        if status_tx.send(status_send).is_err() {
                            warn!("Failed to send exit status: receiver dropped");
                        }
                        return (status, child);
                    },

                    Some(sig) = sig_rx.recv() => {
                        match kill(pid, sig) {
                            Ok(_) => info!("Sent signal {:?} to child process", sig),
                            Err(e) => {
                                error!("Failed to send signal {:?} to child process: {}", sig, e);
                            }
                        }
                    },

                    result = stdout_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => {
                                trace!("stdout: {}", line);
                                if line == READY_LINE {
                                    if status_tx.send(Status::Running).is_err() {
                                        warn!("Failed to send Running status: receiver dropped");
                                    }
                                }
                                stdout_buf.push(line);
                            }
                            Ok(None) => break, // stdout closed
                            Err(e) => {
                                error!("Error reading from stdout: {}", e);
                                break;
                            }
                        }
                    },

                    result = stderr_reader.next_line() => {
                        match result {
                            Ok(Some(line)) => {
                                trace!("stderr: {}", line);
                                stderr_buf.push(line);
                            }
                            Ok(None) => break, // stderr closed
                            Err(e) => {
                                error!("Error reading from stderr: {}", e);
                                break;
                            }
                        }
                    },

                    _ = tokio::time::sleep(STDIO_REPORT_DURATION) => {
                        if !stdout_buf.is_empty() {
                            stdout_rb_.write().await.push_many(stdout_buf.drain(..));
                        }

                        if !stderr_buf.is_empty() {
                            stderr_rb_.write().await.push_many(stderr_buf.drain(..));
                        }
                    }
                }
            }

            let status = child.wait().await;
            arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
            let status_send = match &status {
                Ok(status) => Status::Returned(*status),
                Err(e) => Status::Dead(e.to_string()),
            };
            if status_tx.send(status_send).is_err() {
                warn!("Failed to send final status: receiver dropped");
            }
            (status, child)
        });

        Ok(Self {
            handle,
            pid: arc_pid,
            sig_tx,
            status_rx,
            stdout: stdout_rb,
            stderr: stderr_rb,
        })
    }

    pub async fn stdout_logs(&self) -> Vec<String> {
        self.stdout.read().await.to_vec()
    }

    pub async fn stderr_logs(&self) -> Vec<String> {
        self.stderr.read().await.to_vec()
    }

    pub async fn shutdown(&mut self) -> Result<ExitStatus> {
        let signal = Signal::SIGINT;

        if let Err(Error::ChildReturned(status)) = self.try_ready() {
            return Ok(status);
        }

        let join_result = match self.sig_tx.send(signal).await {
            Ok(_) => tokio::time::timeout(Self::TERM_TIMEOUT_S, &mut self.handle)
                .await
                .map_err(Error::ProcessJoinTimeout)?,
            Err(e) => {
                // should be due to channel close, check if the process is finished
                if !self.handle.is_finished() {
                    return Err(Error::SignalSend(e));
                }
                (&mut self.handle).await
            }
        };

        let (status, child) = join_result.map_err(Error::ProcessJoin)?;

        let status = match status {
            Ok(status) => {
                if status.success() {
                    debug!("RcssServer::shutdown: process exited successfully");
                } else {
                    warn!("RcssServer::shutdown: process exited with status: {status:?}");
                }
                status
            }
            Err(e) => {
                warn!("RcssServer::shutdown: wait on process exits with error, trying KILL...");

                let mut child = child;
                let pid = child.id();

                if let Some(pid) = pid
                    && let Ok(_) = kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
                    && let Ok(status) = child.wait().await
                // todo!("timeout")
                {
                    warn!(
                        "RcssServer::shutdown: process KILLed successfully with pid: {}",
                        pid
                    );
                    status
                } else {
                    return Err(Error::FatalProcessWindingUp {
                        pid,
                        signal,
                        error: e,
                    });
                }
            }
        };

        Ok(status)
    }

    pub fn pid(&self) -> Option<u32> {
        let pid = self.pid.load(std::sync::atomic::Ordering::SeqCst);
        (pid != 0).then_some(pid)
    }

    fn try_ready(&self) -> Result<bool> {
        let status = self.status_rx.borrow().clone();
        match status {
            Status::Dead(e) => Err(Error::ChildDead {
                pid: self.pid(),
                error: e.clone(),
            }),
            Status::Returned(status) => Err(Error::ChildReturned(status)),
            Status::Running => Ok(true),
            Status::Init | Status::Booting => Ok(false),
        }
    }

    /// Wait until the rcssserver is ready and able to accept Udp connections.
    pub async fn until_ready(&mut self, timeout: Option<Duration>) -> Result<()> {
        if self.try_ready()? { return Ok(()) }

        let task = self.status_rx.wait_for(|s| s.is_ready());
        let ret = match timeout {
            Some(timeout) => tokio::time::timeout(timeout, task)
                .await
                .map_err(|_| Error::TimeoutWaitingReady)?,
            None => task.await,
        };

        debug!("RcssServer::until_ready: process status: {:?}", ret);

        if ret.is_ok() {
            debug!("RcssServer::until_ready: process ready to accept Udp conn.");
            return Ok(());
        }

        drop(ret);

        error!("RcssServer::until_ready: UNEXPECTED watch channel released!!!!!");
        Err(Error::ChildNotReady)
    }
}
