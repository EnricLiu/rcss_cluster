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
use tokio::sync::{broadcast, mpsc, watch};
use tokio::task::JoinHandle;
use crate::process::ProcessStatusKind;
use super::status::ProcessStatus;
use super::error::{ProcessError, Result};

pub const TERM_TIMEOUT_S: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub struct Process {
    pid: Arc<AtomicU32>,
    handle: JoinHandle<(Result<ExitStatus>, Child)>,
    sig_tx: mpsc::Sender<Signal>,
    
    // Broadcast channels for stdout/stderr
    stdout_tx: broadcast::Sender<String>,
    stderr_tx: broadcast::Sender<String>,

    status_rx: watch::Receiver<ProcessStatus>,
}

impl Process {
    pub fn new(mut child: Child) -> Result<Self> {
        match child.try_wait() {
            Ok(None) => {},
            Ok(Some(status)) => return Err(ProcessError::ChildAlreadyCompleted(status)),
            Err(e) => return Err(ProcessError::ChildUntrackableWithoutPid(e)),
        }

        let pid = child.id().ok_or(ProcessError::ChildRunningWithoutPid)?;

        let arc_pid = Arc::new(AtomicU32::new(pid));
        let pid = Pid::from_raw(pid as i32);

        let (status_tx, status_rx) = watch::channel(ProcessStatus::init());
        let (sig_tx, mut sig_rx) = mpsc::channel(4);

        let (stdout_tx, _) = broadcast::channel(32);
        let (stderr_tx, _) = broadcast::channel(32);

        let arc_pid_ = Arc::clone(&arc_pid);
        let stdout_tx_ = stdout_tx.clone();
        let stderr_tx_ = stderr_tx.clone();
        
        let handle = tokio::spawn(async move {
            let mut child = child;
            let arc_pid = arc_pid_;

            let mut stdout_reader = {
                let stdout = child.stdout.take().ok_or_else(|| {
                    error!("Failed to capture stdout from child process");
                    io::Error::new(io::ErrorKind::Other, "stdout not available")
                }).expect("stdout should be available with command.stdout(Stdio::piped())");

                BufReader::new(stdout).lines()
            };

            let mut stderr_reader = {
                let stderr = child.stderr.take().ok_or_else(|| {
                    error!("Failed to capture stderr from child process");
                    io::Error::new(io::ErrorKind::Other, "stderr not available")
                }).expect("stderr should be available with command.stdout(Stdio::piped())");

                BufReader::new(stderr).lines()
            };

            // Transition directly to Booting as requested
            status_tx.send_modify(|s| s.as_booting());

            loop {
                tokio::select! {
                    status = child.wait() => {
                        info!("Child process exited with status: {:?}", status);
                        arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
                        status_tx.send_modify(|s| match &status {
                            Ok(status) => s.as_returned(*status),
                            Err(e) => s.as_dead(e.to_string()),
                        });
                        return (status.map_err(ProcessError::Io), child);
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
                                // Broadcast line
                                let _ = stdout_tx_.send(line);
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
                                // Broadcast line
                                let _ = stderr_tx_.send(line);
                            }
                            Ok(None) => break, // stderr closed
                            Err(e) => {
                                error!("Error reading from stderr: {}", e);
                                break;
                            }
                        }
                    },
                }
            }

            let status = child.wait().await;
            arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
            status_tx.send_modify(|s| match &status {
                Ok(status) => s.as_returned(*status),
                Err(e) => s.as_dead(e.to_string()),
            });
            (status.map_err(ProcessError::Io), child)
        });

        Ok(Self {
            handle,
            pid: arc_pid,
            sig_tx,
            status_rx,
            stdout_tx,
            stderr_tx,
        })
    }

    pub fn subscribe_stdout(&self) -> broadcast::Receiver<String> {
        self.stdout_tx.subscribe()
    }

    pub fn subscribe_stderr(&self) -> broadcast::Receiver<String> {
        self.stderr_tx.subscribe()
    }

    pub async fn shutdown(&mut self) -> Result<ExitStatus> {
        let signal = Signal::SIGINT;

        if let Err(ProcessError::ChildReturned(status)) = self.try_ready() {
            return Ok(status);
        }

        let join_result = match self.sig_tx.send(signal).await {
            Ok(_) => tokio::time::timeout(TERM_TIMEOUT_S, &mut self.handle)
                .await
                .map_err(ProcessError::ProcessJoinTimeout)?,
            Err(e) => {
                if !self.handle.is_finished() {
                    return Err(ProcessError::SignalSend(e));
                }
                (&mut self.handle).await
            }
        };

        let (status, child) = join_result.map_err(ProcessError::ProcessJoin)?;

        let status = match status {
            Ok(status) => {
                if status.success() {
                    debug!("Process::shutdown: process exited successfully");
                } else {
                    warn!("Process::shutdown: process exited with status: {status:?}");
                }
                status
            }
            Err(e) => {
                warn!("Process::shutdown: wait on process exits with error, trying KILL... ({})", e);

                let mut child = child;
                let pid = child.id();

                if let Some(pid) = pid {
                     match kill(Pid::from_raw(pid as i32), Signal::SIGKILL) {
                        Ok(_) => {},
                        Err(e) => warn!("Failed to kill process {}: {}", pid, e),
                     }
                     
                     match child.wait().await {
                        Ok(status) => {
                            warn!("Process::shutdown: process KILLed successfully with pid: {}", pid);
                            status
                        },
                        Err(e) => {
                            return Err(ProcessError::FatalProcessWindingUp {
                                pid: Some(pid),
                                signal,
                                error: e,
                            });
                        }
                     }
                } else {
                     return Err(ProcessError::ChildRunningWithoutPid); 
                }
            }
        };

        Ok(status)
    }

    pub fn pid(&self) -> Option<u32> {
        let pid = self.pid.load(std::sync::atomic::Ordering::SeqCst);
        (pid != 0).then_some(pid)
    }

    pub fn status_now(&self) -> ProcessStatus {
        self.status_rx.borrow().clone()
    }
    
    pub fn status_watch(&self) -> watch::Receiver<ProcessStatus> {
        self.status_rx.clone()
    }

    fn try_ready(&self) -> Result<bool> {
        let status = self.status_rx.borrow().clone();
        match status.status() {
            ProcessStatusKind::Dead(e) => Err(ProcessError::ChildDead {
                pid: self.pid(),
                error: e.clone(),
            }),
            ProcessStatusKind::Returned(status) => Err(ProcessError::ChildReturned(status)),
            ProcessStatusKind::Running => Ok(true),
            ProcessStatusKind::Init => Ok(false),
            ProcessStatusKind::Booting => Ok(false),
        }
    }

    pub async fn wait_for_running(&mut self, timeout: Option<Duration>) -> Result<()> {
        if self.try_ready()? { return Ok(()) }

        let task = self.status_rx.wait_for(|s| s.is_ready());
        let ret = match timeout {
            Some(timeout) => tokio::time::timeout(timeout, task)
                .await
                .map_err(|_| ProcessError::TimeoutWaitingReady)?,
            None => task.await,
        };

        debug!("Process::wait_for_running: process status: {:?}", ret);

        if ret.is_ok() {
            debug!("Process::wait_for_running: process ready.");
            return Ok(());
        }
        
        drop(ret);
        error!("Process::wait_for_running: UNEXPECTED watch channel released!!!!!");
        Err(ProcessError::ChildNotReady)
    }
}
