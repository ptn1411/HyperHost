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
            tracing::warn!("nginx already running");
            return Ok(());
        }

        // Ensure required Nginx directories exist
        std::fs::create_dir_all(self.prefix.join("logs"))?;
        std::fs::create_dir_all(self.prefix.join("temp"))?;

        let conf_str = self.conf.to_str().unwrap().replace('\\', "/");
        let prefix_str = self.prefix.to_str().unwrap().replace('\\', "/");

        let child = Command::new(&self.exe)
            .args(["-c", &conf_str, "-p", &prefix_str])
            .spawn()?;

        *proc = Some(child);
        tracing::info!("nginx started");
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

        let _ = Command::new(&self.exe)
            .args(["-p", &prefix_str, "-s", "quit"])
            .status();

        *self.process.lock().unwrap() = None;
        tracing::info!("nginx stopped");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        let mut proc = self.process.lock().unwrap();
        if let Some(mut child) = proc.take() {
            match child.try_wait() {
                Ok(Some(_)) => {
                    false // exited
                }
                Ok(None) => {
                    *proc = Some(child);
                    true // still running
                }
                Err(_) => {
                    false // error checking, assume dead
                }
            }
        } else {
            false
        }
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
}

impl Drop for NginxManager {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
