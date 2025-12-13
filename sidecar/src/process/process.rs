use std::io;
use std::process::ExitStatus;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use std::time::Duration;

use log::{debug, error, info, trace, warn};
use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Child;
use tokio::sync::{mpsc, watch};
use tokio::task::JoinHandle;

use super::builder::ServerProcessSpawner;
use super::*;

pub const READY_LINE: &str = "Hit CTRL-C to exit";

#[derive(Debug)]
pub struct ServerProcess {
    pid: Arc<AtomicU32>,
    handle: JoinHandle<(io::Result<ExitStatus>, Child)>,
    sig_tx: mpsc::Sender<Signal>,
    stdout_rx: mpsc::Receiver<String>,
    stderr_rx: mpsc::Receiver<String>,

    status_rx: watch::Receiver<Status>,
}

impl ServerProcess {
    pub const TERM_TIMEOUT_S: Duration = Duration::from_secs(5);

    pub async fn spawner(pgm_name: &'static str) -> ServerProcessSpawner {
        ServerProcessSpawner::new(pgm_name).await
    }

    pub(crate) async fn try_from(mut child: Child) -> Result<Self> {
        let pid = match child.id() {
            Some(pid) => pid,
            None => match child.try_wait() {
                Ok(Some(status)) => return Err(Error::ChildAlreadyCompleted(status)),
                Ok(None) => return Err(Error::ChildRunningWithoutPid), // todo!("handle it with `child` internally")
                Err(e) => return Err(Error::ChildUntrackableWithoutPid(e)), // todo!("handle it with `child` internally")
            },
        };

        let arc_pid = Arc::new(AtomicU32::new(pid));
        let pid = Pid::from_raw(pid as i32);

        let (status_tx, status_rx) = watch::channel(Status::Init);
        let (sig_tx, mut sig_rx) = mpsc::channel(4);
        let (stdout_tx, stdout_rx) = mpsc::channel(32);
        let (stderr_tx, stderr_rx) = mpsc::channel(32);

        let arc_pid_ = Arc::clone(&arc_pid);
        let handle = tokio::spawn(async move {
            let mut child = child;
            let arc_pid = arc_pid_;
            let stdout = child.stdout.take().expect("Failed to open stdout");
            let stderr = child.stderr.take().expect("Failed to open stderr");

            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            status_tx
                .send(Status::Booting)
                .expect("Failed to send status");

            loop {
                tokio::select! {
                    status = child.wait() => {
                        info!("RcssServer child process exited with status: {:?}", status);
                        arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
                        let status_send = match &status {
                            Ok(status) => Status::Returned(*status),
                            Err(e) => Status::Dead(e.to_string()),
                        };
                        status_tx.send(status_send).expect("Failed to send status");
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
                                trace!("stdout: {}", line);
                                if line == READY_LINE {
                                    status_tx.send(Status::Running).expect("Failed to send status");
                                }
                                // if stdout_tx.send(line).await.is_err() {
                                //     // Channel closed, probably the receiver was dropped.
                                //     break;
                                // }
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
                                trace!("stderr: {}", line);
                                // if stderr_tx.send(line).await.is_err() {
                                //     break;
                                // }
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
            let status_send = match &status {
                Ok(status) => Status::Returned(*status),
                Err(e) => Status::Dead(e.to_string()),
            };
            status_tx.send(status_send).expect("Failed to send status");
            (status, child)
        });

        Ok(Self {
            handle,
            pid: arc_pid,
            sig_tx,
            status_rx,
            stdout_rx,
            stderr_rx,
        })
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
        if self.try_ready()? {
            return Ok(());
        }

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
