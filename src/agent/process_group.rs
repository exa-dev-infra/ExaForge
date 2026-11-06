use std::{
    path::Path,
    process::Stdio,
    time::Duration,
};
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::watch,
};
use tracing::{info, warn};

use crate::pkg::errors::AppError;

pub struct ProcessGroupExecutor;

impl ProcessGroupExecutor {
    /// Spawns a shell script inside a dedicated OS process group with clean termination.
    pub async fn execute_command<F>(
        command: &str,
        working_dir: &Path,
        env_vars: &[(String, String)],
        timeout: Duration,
        mut cancel_rx: watch::Receiver<bool>,
        log_callback: F,
    ) -> Result<i32, AppError>
    where
        F: Fn(String, bool) + Send + Sync + 'static, // (line, is_stderr)
    {
        let mut cmd = Command::new("/bin/sh");
        cmd.arg("-c")
            .arg(command)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (k, v) in env_vars {
            cmd.env(k, v);
        }

        // Set process group leader so child.pid == pgid
        #[cfg(unix)]
        unsafe {
            cmd.pre_exec(|| {
                nix::unistd::setpgid(Pid::from_raw(0), Pid::from_raw(0))
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                Ok(())
            });
        }

        let mut child = cmd.spawn()
            .map_err(|e| AppError::internal(format!("Failed to spawn process group: {}", e)))?;

        let child_pid = child.id().ok_or_else(|| AppError::internal("Failed to retrieve child PID".to_string()))?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();
        let log_cb_out = std::sync::Arc::new(log_callback);
        let log_cb_err = log_cb_out.clone();

        let stdout_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                log_cb_out(line, false);
            }
        });

        let stderr_task = tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                log_cb_err(line, true);
            }
        });

        let wait_result = tokio::select! {
            res = child.wait() => {
                match res {
                    Ok(status) => status.code().unwrap_or(-1),
                    Err(e) => {
                        warn!(pid = child_pid, error = %e, "Process wait failed");
                        -1
                    }
                }
            }
            _ = tokio::time::sleep(timeout) => {
                warn!(pid = child_pid, "Process execution timed out! Killing process group tree (-pgid)...");
                Self::kill_process_group(child_pid as i32).await;
                -9
            }
            _ = async {
                while cancel_rx.changed().await.is_ok() {
                    if *cancel_rx.borrow() {
                        break;
                    }
                }
            } => {
                info!(pid = child_pid, "Cancellation signal received! Killing process group tree (-pgid)...");
                Self::kill_process_group(child_pid as i32).await;
                -15
            }
        };

        let _ = stdout_task.await;
        let _ = stderr_task.await;

        Ok(wait_result)
    }

    /// Kills the entire process group (-pgid) with SIGTERM, then SIGKILL
    pub async fn kill_process_group(pgid: i32) {
        #[cfg(unix)]
        {
            let target_pid = Pid::from_raw(-pgid);
            let _ = kill(target_pid, Signal::SIGTERM);

            tokio::time::sleep(Duration::from_millis(1500)).await;

            let _ = kill(target_pid, Signal::SIGKILL);
            info!(pgid = pgid, "Process group terminated cleanly via SIGKILL");
        }
    }
}
