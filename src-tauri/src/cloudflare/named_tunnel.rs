use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

pub struct NamedTunnelManager {
    processes: Arc<Mutex<HashMap<String, Child>>>,
}

impl NamedTunnelManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Run `cloudflared tunnel login` — opens browser for Cloudflare auth.
    /// Blocks until the user completes login (cert.pem created).
    pub fn login(cloudflared_exe: &Path) -> anyhow::Result<()> {
        let status = Command::new(cloudflared_exe)
            .args(["tunnel", "login"])
            .status()?;
        if !status.success() {
            anyhow::bail!("cloudflared tunnel login failed");
        }
        Ok(())
    }

    /// Check whether the user is already logged in (cert.pem exists).
    pub fn is_logged_in() -> bool {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map(PathBuf::from)
            .unwrap_or_default();
        home.join(".cloudflared").join("cert.pem").exists()
    }

    /// Run `cloudflared tunnel create <name>`.
    /// Returns (tunnel_id, credentials_path).
    pub fn create_tunnel(cloudflared_exe: &Path, name: &str) -> anyhow::Result<(String, String)> {
        let output = Command::new(cloudflared_exe)
            .args(["tunnel", "create", name])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let combined = format!("{}\n{}", stdout, stderr);

        if !output.status.success() {
            anyhow::bail!("cloudflared tunnel create failed:\n{}", combined);
        }

        // Parse: "Created tunnel <name> with id <UUID>"
        let tunnel_id = combined
            .lines()
            .find_map(|line| {
                if line.contains("with id ") {
                    line.split("with id ").nth(1).map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Could not parse tunnel ID from output:\n{}", combined)
            })?;

        // Parse: "Tunnel credentials written to <path>"
        let creds_path = combined
            .lines()
            .find_map(|line| {
                if line.contains("credentials written to ") {
                    line.split("credentials written to ")
                        .nth(1)
                        .map(|s| s.trim().trim_end_matches('.').to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| {
                anyhow::anyhow!("Could not parse credentials path from output:\n{}", combined)
            })?;

        tracing::info!(
            "Created named tunnel '{}' id={} creds={}",
            name,
            tunnel_id,
            creds_path
        );
        Ok((tunnel_id, creds_path))
    }

    /// Generate a config.yml file for the named tunnel.
    /// `routes` is a list of (hostname, upstream) pairs.
    pub fn generate_config(
        config_path: &Path,
        tunnel_id: &str,
        credentials_path: &str,
        routes: &[(String, String)],
    ) -> anyhow::Result<()> {
        let creds_fwd = credentials_path.replace('\\', "/");
        let mut content = format!(
            "tunnel: {tunnel_id}\ncredentials-file: \"{creds}\"\n\ningress:\n",
            tunnel_id = tunnel_id,
            creds = creds_fwd,
        );

        for (hostname, upstream) in routes {
            content.push_str(&format!(
                "  - hostname: {}\n    service: {}\n",
                hostname, upstream
            ));
        }
        content.push_str("  - service: http_status:404\n");

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(config_path, content)?;
        tracing::info!("Generated tunnel config at {}", config_path.display());
        Ok(())
    }

    /// Start the named tunnel using its config.yml.
    pub fn start(
        &self,
        name: &str,
        config_path: &Path,
        cloudflared_exe: &Path,
    ) -> anyhow::Result<()> {
        let mut procs = self.processes.lock().unwrap();
        if procs.contains_key(name) {
            tracing::warn!("Named tunnel '{}' already running", name);
            return Ok(());
        }

        let child = Command::new(cloudflared_exe)
            .args([
                "tunnel",
                "--config",
                config_path.to_str().unwrap(),
                "run",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        procs.insert(name.to_string(), child);
        tracing::info!("Named tunnel '{}' started", name);
        Ok(())
    }

    /// Stop a running named tunnel.
    pub fn stop(&self, name: &str) {
        let mut procs = self.processes.lock().unwrap();
        if let Some(mut child) = procs.remove(name) {
            let _ = child.kill();
            let _ = child.wait();
            tracing::info!("Named tunnel '{}' stopped", name);
        }
    }

    /// Check if a named tunnel process is currently running.
    pub fn is_running(&self, name: &str) -> bool {
        let mut procs = self.processes.lock().unwrap();
        if let Some(child) = procs.get_mut(name) {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    procs.remove(name);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn stop_all(&self) {
        let mut procs = self.processes.lock().unwrap();
        for (name, mut child) in procs.drain() {
            let _ = child.kill();
            let _ = child.wait();
            tracing::info!("Stopped named tunnel '{}'", name);
        }
    }
}

impl Drop for NamedTunnelManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}
