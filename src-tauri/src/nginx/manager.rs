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
    /// Validates config with `nginx -t` first to prevent downtime on bad config.
    pub fn reload(&self) -> anyhow::Result<()> {
        if !self.is_running() {
            tracing::info!("nginx not running, reload skipped");
            return Ok(());
        }

        // Pre-flight: validate config before applying
        self.test_config().map_err(|e| {
            anyhow::anyhow!("nginx config validation failed, reload aborted:\n{}", e)
        })?;

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
        if !self.is_running() {
            return Ok(());
        }

        let conf_str = self.conf.to_str().unwrap().replace('\\', "/");
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        // Graceful quit — must pass -c so nginx can locate its PID file
        let _ = Command::new(&self.exe)
            .args(["-c", &conf_str, "-p", &prefix_str, "-s", "quit"])
            .status();

        // Give it a moment to shut down
        std::thread::sleep(std::time::Duration::from_millis(300));

        // If PID file still exists, the graceful quit failed — force kill
        if let Some(pid) = self.read_pid_file() {
            if Self::is_pid_alive(pid) {
                tracing::warn!("nginx did not quit gracefully, force killing pid={}", pid);
                Self::force_kill(pid);
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
        let conf_str = self.conf.to_str().unwrap().replace('\\', "/");
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        let pid_opt = self.read_pid_file();
        let is_alive = pid_opt.map(|pid| Self::is_pid_alive(pid)).unwrap_or(false);

        if is_alive {
            // 1. Try graceful stop via nginx signal — only if conf file exists,
            //    otherwise nginx will error trying to read {prefix}/conf/nginx.conf
            let quit_result = if self.conf.exists() {
                Command::new(&self.exe)
                    .args(["-c", &conf_str, "-p", &prefix_str, "-s", "quit"])
                    .output()
            } else {
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "conf not yet written"))
            };

            if let Ok(output) = &quit_result {
                if output.status.success() {
                    tracing::info!("Sent quit signal to stale nginx");
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }

        // 2. If PID file still exists, force kill that specific PID
        if let Some(pid) = self.read_pid_file() {
            if Self::is_pid_alive(pid) {
                tracing::warn!("Force killing stale nginx pid={}", pid);
                Self::force_kill(pid);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }

        // 3. Clean up stale PID file
        let _ = std::fs::remove_file(self.pid_file());
    }

    /// Check if a process with the given PID is still alive.
    fn is_pid_alive(pid: u32) -> bool {
        #[cfg(target_os = "windows")]
        {
            Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/NH"])
                .output()
                .map(|out| {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    !stdout.contains("No tasks") && stdout.contains(&pid.to_string())
                })
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        {
            // `kill -0 <pid>` succeeds if process exists and we have permission
            Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        }
    }

    /// Force-kill a process by PID.
    fn force_kill(pid: u32) {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output();
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = Command::new("kill")
                .args(["-9", &pid.to_string()])
                .output();
        }
    }
}

impl Drop for NginxManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
