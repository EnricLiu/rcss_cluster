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
use tokio::sync::{mpsc, watch, RwLock};
use tokio::task::JoinHandle;

use common::utils::ringbuf::OverwriteRB;
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

    status_rx: watch::Receiver<Status>,
}

impl ServerProcess {
    pub const TERM_TIMEOUT_S: Duration = Duration::from_secs(5);

    pub async fn spawner(pgm_name: &'static str) -> ServerProcessSpawner {
        ServerProcessSpawner::new(pgm_name).await
    }

    pub(crate) async fn try_from(mut child: Child) -> Result<ServerProcess> {
        match child.try_wait() {
            Ok(None) => {},
            Ok(Some(status)) => return Err(Error::ChildAlreadyCompleted(status)),
            Err(e) => return Err(Error::ChildUntrackableWithoutPid(e)), // todo!("handle it with `child` internally")
        }

        let pid = child.id().ok_or(Error::ChildRunningWithoutPid)?;

        let arc_pid = Arc::new(AtomicU32::new(pid));
        let pid = Pid::from_raw(pid as i32);

        let (status_tx, status_rx) = watch::channel(Status::init());
        let (sig_tx, mut sig_rx) = mpsc::channel(4);
        let stdout_rb = status_rx.borrow().stdout.clone();
        let stderr_rb = status_rx.borrow().stderr.clone();

        let arc_pid_ = Arc::clone(&arc_pid);
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

            status_tx.send_modify(|s| s.as_booting());

            let mut stdout_buf = Vec::with_capacity(32);
            let mut stderr_buf = Vec::with_capacity(8);

            loop {
                tokio::select! {
                    status = child.wait() => {
                        info!("RcssServer child process exited with status: {:?}", status);
                        arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
                        status_tx.send_modify(|s| match &status {
                            Ok(status) => s.as_returned(*status),
                            Err(e) => s.as_dead(e.to_string()),
                        });

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
                                    status_tx.send_modify(|s| s.as_running())
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
                            stdout_rb.write().await.push_many(stdout_buf.drain(..));
                        }

                        if !stderr_buf.is_empty() {
                            stderr_rb.write().await.push_many(stderr_buf.drain(..));
                        }
                    }
                }
            }

            let status = child.wait().await;

            if !stdout_buf.is_empty() {
                stdout_rb.write().await.push_many(stdout_buf.drain(..));
            }
            if !stderr_buf.is_empty() {
                stderr_rb.write().await.push_many(stderr_buf.drain(..));
            }

            arc_pid.store(0, std::sync::atomic::Ordering::SeqCst);
            status_tx.send_modify(|s| match &status {
                Ok(status) => s.as_returned(*status),
                Err(e) => s.as_dead(e.to_string()),
            });

            (status, child)
        });

        Ok(Self {
            handle,
            pid: arc_pid,
            sig_tx,
            status_rx,
        })
    }

    pub fn status_now(&self) -> Status {
        self.status_rx.borrow().clone()
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
        match status.status() {
            StatusKind::Dead(e) => Err(Error::ChildDead {
                pid: self.pid(),
                error: e.clone(),
            }),
            StatusKind::Returned(status) => Err(Error::ChildReturned(status)),
            StatusKind::Running => Ok(true),
            StatusKind::Init | StatusKind::Booting => Ok(false),
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

    pub fn status_watch(&self) -> watch::Receiver<Status> {
        self.status_rx.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use tokio::process::Command;
    use std::time::Duration;

    // Helper function to create a test child process that echoes and exits
    async fn create_test_child(script: &str) -> Child {
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to spawn test child process")
    }

    #[tokio::test]
    async fn test_server_process_creation_with_valid_child() {
        // Create a simple child process that prints and exits
        let child = create_test_child("echo 'test'; sleep 0.1").await;

        let result = ServerProcess::try_from(child).await;

        assert!(result.is_ok(), "Should successfully create ServerProcess from valid child");
        let mut process = result.unwrap();

        // Verify PID is set
        assert!(process.pid().is_some(), "PID should be set");

        // Clean up
        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_server_process_pid_tracking() {
        let child = create_test_child("sleep 0.5").await;
        let pid_before = child.id().expect("Child should have PID");

        let process = ServerProcess::try_from(child).await.unwrap();

        // PID should match
        assert_eq!(process.pid(), Some(pid_before));

        // After shutdown, PID should be cleared
        let mut process = process;
        let _ = process.shutdown().await;

        // Give it a moment to update
        tokio::time::sleep(Duration::from_millis(50)).await;
        assert_eq!(process.pid(), None, "PID should be cleared after process exits");
    }

    #[tokio::test]
    async fn test_stdout_capture() {
        let script = r#"
            echo "line1"
            echo "line2"
            echo "line3"
            sleep 1
        "#;
        let child = create_test_child(script).await;

        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Wait for logs to be captured (STDIO_REPORT_DURATION is 500ms)
        tokio::time::sleep(Duration::from_millis(600)).await;

        let logs = process.status_now().stdout_logs().await;

        assert!(!logs.is_empty(), "Should capture stdout logs");
        assert!(logs.contains(&"line1".to_string()), "Should contain line1");
        assert!(logs.contains(&"line2".to_string()), "Should contain line2");
        assert!(logs.contains(&"line3".to_string()), "Should contain line3");

        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_stderr_capture() {
        let script = r#"
            echo "error1" >&2
            echo "error2" >&2
            sleep 1
        "#;
        let child = create_test_child(script).await;

        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Wait for logs to be captured
        tokio::time::sleep(Duration::from_millis(600)).await;

        let logs = process.status_now().stderr_logs().await;

        assert!(!logs.is_empty(), "Should capture stderr logs");
        assert!(logs.contains(&"error1".to_string()), "Should contain error1");
        assert!(logs.contains(&"error2".to_string()), "Should contain error2");

        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_ready_line_detection() {
        let script = format!(r#"
            echo "Starting..."
            sleep 0.2
            echo "{}"
            sleep 2
        "#, READY_LINE);

        let child = create_test_child(&script).await;
        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Wait for ready with timeout
        let result = process.until_ready(Some(Duration::from_secs(2))).await;

        assert!(result.is_ok(), "Process should become ready when READY_LINE is printed");

        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_until_ready_timeout() {
        // Process that never prints the ready line
        let child = create_test_child("sleep 5").await;
        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Should timeout
        let result = process.until_ready(Some(Duration::from_millis(100))).await;

        assert!(result.is_err(), "Should timeout when ready line is not printed");
        assert!(matches!(result.unwrap_err(), Error::TimeoutWaitingReady));

        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        let child = create_test_child("exec sleep 10").await;
        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Give process time to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        let result = process.shutdown().await;
        println!("{result:?}");

        assert!(result.is_ok(), "Graceful shutdown should succeed");

        // PID should be cleared
        assert_eq!(process.pid(), None, "PID should be cleared after shutdown");
    }

    #[tokio::test]
    async fn test_shutdown_already_exited_process() {
        // Process that exits immediately
        let child = create_test_child("exit 0").await;
        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Wait for process to exit
        tokio::time::sleep(Duration::from_millis(200)).await;

        let result = process.shutdown().await;

        // Should handle already-exited process gracefully
        assert!(result.is_ok(), "Should handle already-exited process");
    }

    #[tokio::test]
    async fn test_ringbuf_overflow_stdout() {
        // Generate more than 32 lines (the ring buffer capacity)
        let mut script = String::from("#!/bin/sh\n");
        for i in 0..50 {
            script.push_str(&format!("echo 'line{}'\n", i));
        }
        script.push_str("sleep 1\n");

        let child = create_test_child(&script).await;
        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Wait for all logs to be captured
        tokio::time::sleep(Duration::from_millis(800)).await;

        let logs = process.status_now().stdout_logs().await;

        // Ring buffer should contain at most 32 entries
        assert!(logs.len() <= 32, "Ring buffer should not exceed capacity of 32");

        // Should contain the most recent lines
        assert!(logs.contains(&"line49".to_string()), "Should contain the last line");

        // Should NOT contain the earliest lines (they were overwritten)
        assert!(!logs.contains(&"line0".to_string()), "Earliest lines should be overwritten");

        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_try_from_already_completed_child() {
        // Create a child that exits immediately
        let child = create_test_child("exit 1").await;

        // Wait for it to complete
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Try to create ServerProcess from completed child
        let result = ServerProcess::try_from(child).await;
        println!("{result:?}");

        assert!(result.is_err(), "Should fail when child is already completed");
        assert!(matches!(result.unwrap_err(), Error::ChildAlreadyCompleted(_)));
    }

    #[tokio::test]
    async fn test_status_transitions() {
        let script = format!(r#"
            echo "booting"
            sleep 0.2
            echo "{}"
            sleep 2
        "#, READY_LINE);

        let child = create_test_child(&script).await;
        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Initial status should be Init or Booting
        let initial_status = process.status_rx.borrow().clone();
        assert!(matches!(initial_status.status(), StatusKind::Init | StatusKind::Booting));

        // Wait for ready
        let _ = process.until_ready(Some(Duration::from_secs(2))).await;

        // Status should now be Running
        let running_status = process.status_rx.borrow().clone();
        assert!(matches!(running_status.status(), StatusKind::Running));

        // Shutdown
        let exit_status = process.shutdown().await.unwrap();

        // Status should be Returned
        let final_status = process.status_rx.borrow().clone();
        assert!(matches!(final_status.status(), StatusKind::Returned(_)));
    }

    #[tokio::test]
    async fn test_concurrent_stdout_stderr() {
        let script = r#"
            echo "stdout1"
            echo "stderr1" >&2
            echo "stdout2"
            echo "stderr2" >&2
            sleep 1
        "#;

        let child = create_test_child(script).await;
        let mut process = ServerProcess::try_from(child).await.unwrap();

        // Wait for logs
        tokio::time::sleep(Duration::from_millis(600)).await;

        let status = process.status_now();
        let stdout_logs = status.stdout_logs().await;
        let stderr_logs = status.stderr_logs().await;

        // Both should have captured their respective streams
        assert!(stdout_logs.contains(&"stdout1".to_string()));
        assert!(stdout_logs.contains(&"stdout2".to_string()));
        assert!(stderr_logs.contains(&"stderr1".to_string()));
        assert!(stderr_logs.contains(&"stderr2".to_string()));

        // Stdout should not contain stderr and vice versa
        assert!(!stdout_logs.contains(&"stderr1".to_string()));
        assert!(!stderr_logs.contains(&"stdout1".to_string()));

        let _ = process.shutdown().await;
    }
}
