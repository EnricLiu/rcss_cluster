use std::process::ExitStatus;
use std::sync::Arc;
use std::time::Duration;
use common::process::{Process, ProcessError, ProcessStatus as Status, ProcessStatusKind};
use common::utils::ringbuf::OverwriteRB;
use tokio::process::Child;
use tokio::sync::{watch, RwLock};
use log::{warn, error};

use super::builder::ServerProcessSpawner;
use super::error::{Error, Result};

pub const READY_LINE: &str = "Hit CTRL-C to exit";

#[derive(Debug)]
pub struct ServerProcess {
    inner: Process,
    status_rx: watch::Receiver<Status>,
}

impl ServerProcess {
    pub const TERM_TIMEOUT_S: Duration = Duration::from_secs(5);

    pub async fn spawner(pgm_name: &'static str) -> ServerProcessSpawner {
        ServerProcessSpawner::new(pgm_name).await
    }

    pub(crate) async fn try_from(child: Child) -> Result<ServerProcess> {
        let inner = Process::new(child)?;

        let (status_tx, status_rx) = watch::channel(Status::init());
        let stdout_rb = status_rx.borrow().stdout.clone();
        let stderr_rb = status_rx.borrow().stderr.clone();

        let mut stdout_rx = inner.subscribe_stdout();
        let mut stderr_rx = inner.subscribe_stderr();

        let mut inner_status_rx = inner.status_watch();

        status_tx.send_modify(|s| s.as_booting());

        const STDOUT_BUF_CAPACITY: usize = 32;
        const STDERR_BUF_CAPACITY: usize = 8;
        let mut stdout_buf = Vec::with_capacity(STDOUT_BUF_CAPACITY);
        let mut stderr_buf = Vec::with_capacity(STDERR_BUF_CAPACITY);

        // Task to process logs and update status
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(line) = stdout_rx.recv() => {
                        if line == READY_LINE {
                            status_tx.send_modify(|s| s.as_running())
                        }
                        stdout_buf.push(line);
                        if stdout_buf.len() >= STDOUT_BUF_CAPACITY {
                            stdout_rb.write().await.push_many(stdout_buf.drain(..));
                        }
                    }
                    Ok(line) = stderr_rx.recv() => {
                        stderr_buf.push(line);
                        if stderr_buf.len() >= STDERR_BUF_CAPACITY {
                            stderr_rb.write().await.push_many(stderr_buf.drain(..));
                        }
                    }

                    _ = inner_status_rx.changed() => {
                        let inner = inner_status_rx.borrow().clone();
                        if inner.is_finished() {
                            status_tx.send_modify(|s| {
                                s.kind = inner.kind.clone();
                            });
                            break
                        }
                    },
                }
            }

            if !stdout_buf.is_empty() {
                stdout_rb.write().await.push_many(stdout_buf.drain(..));
            }
            if !stderr_buf.is_empty() {
                stderr_rb.write().await.push_many(stderr_buf.drain(..));
            }
        });

        Ok(Self {
            inner,
            status_rx,
        })
    }

    pub fn status_now(&self) -> Status {
        self.status_rx.borrow().clone()
    }

    pub async fn shutdown(&mut self) -> Result<ExitStatus> {
        self.inner.shutdown().await.map_err(Error::from)
    }

    pub fn pid(&self) -> Option<u32> {
        self.inner.pid()
    }

    pub fn status_watch(&self) -> watch::Receiver<Status> {
        self.status_rx.clone()
    }

    /// Wait until the rcssserver is ready and able to accept Udp connections.
    pub async fn until_ready(&mut self, timeout: Option<Duration>) -> Result<()> {
         if self.status_rx.borrow().is_ready() { return Ok(()) }

        let task = self.status_rx.wait_for(|s| s.is_ready());
        let ret = match timeout {
            Some(timeout) => tokio::time::timeout(timeout, task).await
                .map_err(|_| Error::Process(ProcessError::TimeoutWaitingReady))?,
            None => task.await,
        };

        if ret.is_ok() {
            return Ok(());
        }

        drop(ret);
        error!("ServerProcess::until_ready: UNEXPECTED watch channel released!!!!!");
        Err(Error::Process(ProcessError::ChildNotReady))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use tokio::process::Command;
    use std::time::Duration;
    use common::process::ProcessError;

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
        let child = create_test_child("echo 'test'; sleep 0.1").await;
        let result = ServerProcess::try_from(child).await;
        assert!(result.is_ok(), "Should successfully create ServerProcess from valid child");
        let mut process = result.unwrap();
        assert!(process.pid().is_some(), "PID should be set");
        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_server_process_pid_tracking() {
        let child = create_test_child("sleep 0.5").await;
        let pid_before = child.id().expect("Child should have PID");
        let mut process = ServerProcess::try_from(child).await.unwrap();
        assert_eq!(process.pid(), Some(pid_before));
        let _ = process.shutdown().await;
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
        let result = process.until_ready(Some(Duration::from_secs(2))).await;
        assert!(result.is_ok(), "Process should become ready when READY_LINE is printed");
        let _ = process.shutdown().await;
    }

    #[tokio::test]
    async fn test_until_ready_timeout() {
        let child = create_test_child("sleep 5").await;
        let mut process = ServerProcess::try_from(child).await.unwrap();
        let result = process.until_ready(Some(Duration::from_millis(100))).await;
        match result.unwrap_err() {
            Error::Process(ProcessError::TimeoutWaitingReady) => {},
            e => panic!("Unexpected error: {:?}", e),
        }
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
    async fn test_terminal_status_propagation() {
        // Process that exits with code 0
        let child = create_test_child("exit 0").await;
        let process = ServerProcess::try_from(child).await.unwrap();

        let mut watch = process.status_watch();

        // Wait for the status to reach a finished state
        let result = tokio::time::timeout(
            Duration::from_secs(2),
            watch.wait_for(|s| s.is_finished()),
        ).await;

        assert!(result.is_ok(), "Should not time out waiting for terminal status");
        let status_ref = result.unwrap();
        assert!(status_ref.is_ok(), "Watch channel should deliver terminal status, not close");
        let status = status_ref.unwrap().clone();
        assert!(status.is_finished(), "Status should be finished");
        match status.kind {
            ProcessStatusKind::Returned(exit_status) => {
                assert!(exit_status.success(), "Process should have exited successfully");
            }
            other => panic!("Expected Returned status, got {:?}", other),
        }
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
}
