use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Mutex;

pub struct NginxManager {
    process: Mutex<Option<Child>>,
    pub exe: PathBuf,
    pub conf: PathBuf,
    pub prefix: PathBuf,
}

impl NginxManager {
    pub fn new(exe: PathBuf, conf: PathBuf, prefix: PathBuf) -> Self {
        Self {
            process: Mutex::new(None),
            exe,
            conf,
            prefix,
        }
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let mut proc = self.process.lock().unwrap();
        if proc.is_some() {
            tracing::warn!("nginx already running (in-memory handle exists)");
            return Ok(());
        }

        // Kill any stale nginx from previous DevHost runs
        self.kill_stale_processes();

        // Ensure required Nginx directories exist
        std::fs::create_dir_all(self.prefix.join("logs"))?;
        std::fs::create_dir_all(self.prefix.join("temp"))?;

        let conf_str = self.conf.to_str().unwrap().replace('\\', "/");
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        let child = Command::new(&self.exe)
            .args(["-c", &conf_str, "-p", &prefix_str])
            .spawn()?;

        *proc = Some(child);
        tracing::info!("nginx started (pid={})", proc.as_ref().unwrap().id());
        Ok(())
    }

    /// Zero-downtime config reload.
    pub fn reload(&self) -> anyhow::Result<()> {
        let conf_str = self.conf.to_str().unwrap().replace('\\', "/");
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        let status = Command::new(&self.exe)
            .args(["-c", &conf_str, "-p", &prefix_str, "-s", "reload"])
            .status()?;

        if !status.success() {
            anyhow::bail!("nginx reload failed with status: {}", status);
        }

        tracing::info!("nginx config reloaded");
        Ok(())
    }

    pub fn stop(&self) -> anyhow::Result<()> {
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        // Graceful quit
        let _ = Command::new(&self.exe)
            .args(["-p", &prefix_str, "-s", "quit"])
            .status();

        // Give it a moment to shut down
        std::thread::sleep(std::time::Duration::from_millis(300));

        // If PID file still exists, the graceful quit failed — force kill
        if let Some(pid) = self.read_pid_file() {
            if Self::is_pid_alive(pid) {
                tracing::warn!("nginx did not quit gracefully, force killing pid={}", pid);
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .output();
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }

        // Clean up PID file
        let _ = std::fs::remove_file(self.pid_file());

        *self.process.lock().unwrap() = None;
        tracing::info!("nginx stopped");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        // Check in-memory child process first
        let mut proc = self.process.lock().unwrap();
        if let Some(mut child) = proc.take() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    return false; // exited
                }
                Ok(None) => {
                    *proc = Some(child);
                    return true; // still running
                }
                Err(_) => {
                    return false;
                }
            }
        }

        // Fallback: check PID file (covers processes from previous DevHost runs)
        if let Some(pid) = self.read_pid_file() {
            return Self::is_pid_alive(pid);
        }

        false
    }

    /// Test the current config for syntax errors.
    pub fn test_config(&self) -> anyhow::Result<String> {
        let conf_str = self.conf.to_str().unwrap().replace('\\', "/");
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        let output = Command::new(&self.exe)
            .args(["-c", &conf_str, "-p", &prefix_str, "-t"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if output.status.success() {
            Ok(stderr)
        } else {
            anyhow::bail!("nginx config test failed:\n{}", stderr)
        }
    }

    // ── Private helpers ──

    fn pid_file(&self) -> PathBuf {
        self.prefix.join("nginx.pid")
    }

    fn read_pid_file(&self) -> Option<u32> {
        std::fs::read_to_string(self.pid_file())
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
    }

    /// Kill stale nginx processes from previous DevHost runs.
    /// Only targets OUR nginx instance (by prefix), not other nginx on the system.
    fn kill_stale_processes(&self) {
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        // 1. Try graceful stop via nginx signal (targets our prefix only)
        let quit_result = Command::new(&self.exe)
            .args(["-p", &prefix_str, "-s", "quit"])
            .output();

        if let Ok(output) = &quit_result {
            if output.status.success() {
                tracing::info!("Sent quit signal to stale nginx");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        // 2. If PID file still exists, force kill that specific PID
        if let Some(pid) = self.read_pid_file() {
            if Self::is_pid_alive(pid) {
                tracing::warn!("Force killing stale nginx pid={}", pid);
                // /T flag kills the entire process tree (master + workers)
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .output();
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        // 3. Clean up stale PID file
        let _ = std::fs::remove_file(self.pid_file());
    }

    /// Check if a process with the given PID is still alive (Windows).
    fn is_pid_alive(pid: u32) -> bool {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output()
            .map(|out| {
                let stdout = String::from_utf8_lossy(&out.stdout);
                // tasklist prints "INFO: No tasks..." when PID doesn't exist
                !stdout.contains("No tasks") && stdout.contains(&pid.to_string())
            })
            .unwrap_or(false)
    }
}

impl Drop for NginxManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
