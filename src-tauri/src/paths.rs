use std::path::PathBuf;

/// All HyperHost data lives under a platform-appropriate directory:
/// - Windows : `%LOCALAPPDATA%\HyperHost`  (falls back to `DevHost` for existing installs)
/// - macOS   : `~/Library/Application Support/HyperHost`
/// - Linux   : `~/.config/hyperhost`
pub struct AppPaths {
    base: PathBuf,
}

impl AppPaths {
    pub fn new() -> Self {
        let base = resolve_base_dir().unwrap_or_else(|| PathBuf::from(".hyperhost"));
        Self { base }
    }

    pub fn base_dir(&self) -> &PathBuf {
        &self.base
    }
    pub fn db_path(&self) -> PathBuf {
        // Support legacy devhost.db for existing Windows installations
        let legacy = self.base.join("devhost.db");
        if legacy.exists() {
            return legacy;
        }
        self.base.join("hyperhost.db")
    }
    pub fn ca_cert(&self) -> PathBuf {
        self.base.join("ca.crt")
    }
    pub fn ca_key(&self) -> PathBuf {
        self.base.join("ca.key")
    }
    pub fn cert_dir(&self) -> PathBuf {
        self.base.join("certs")
    }
    pub fn nginx_dir(&self) -> PathBuf {
        self.base.join("nginx")
    }
    pub fn nginx_conf(&self) -> PathBuf {
        self.base.join("nginx").join("nginx.conf")
    }
    pub fn nginx_logs(&self) -> PathBuf {
        self.base.join("nginx").join("logs")
    }
    pub fn nginx_conf_subdir(&self) -> PathBuf {
        self.base.join("nginx").join("conf")
    }
    pub fn cloudflared_dir(&self) -> PathBuf {
        self.base.join("cloudflared")
    }
    pub fn tunnel_config(&self, tunnel_name: &str) -> PathBuf {
        self.cloudflared_dir().join(format!("{}.yml", tunnel_name))
    }
    pub fn log_path(&self) -> PathBuf {
        self.base.join("hyperhost.log")
    }

    /// Ensure all required directories exist and seed static nginx config files.
    pub fn ensure_dirs(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.cert_dir())?;
        std::fs::create_dir_all(self.nginx_dir())?;
        std::fs::create_dir_all(self.nginx_logs())?;
        std::fs::create_dir_all(self.nginx_conf_subdir())?;
        std::fs::create_dir_all(self.cloudflared_dir())?;

        // Write mime.types if not already present — nginx requires this file
        // via the `include "{nginx_dir}/conf/mime.types"` directive in nginx.conf
        let mime_types_path = self.nginx_conf_subdir().join("mime.types");
        if !mime_types_path.exists() {
            std::fs::write(
                &mime_types_path,
                include_str!("../binaries/nginx-extracted/nginx-1.26.2/conf/mime.types"),
            )?;
        }

        Ok(())
    }
}

fn resolve_base_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let base = dirs::data_local_dir()?;
        // Backward compat: use DevHost/ if it already exists (existing installs)
        let legacy = base.join("DevHost");
        if legacy.exists() {
            tracing::info!("Using legacy DevHost data dir: {}", legacy.display());
            return Some(legacy);
        }
        Some(base.join("HyperHost"))
    }

    #[cfg(target_os = "macos")]
    {
        // ~/Library/Application Support/HyperHost
        Some(dirs::data_local_dir()?.join("HyperHost"))
    }

    #[cfg(target_os = "linux")]
    {
        // ~/.config/hyperhost
        Some(dirs::config_dir()?.join("hyperhost"))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Some(PathBuf::from(".hyperhost"))
    }
}
