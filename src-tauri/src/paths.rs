use std::path::PathBuf;

/// All DevHost data lives under `%LOCALAPPDATA%\DevHost\`.
pub struct AppPaths {
    base: PathBuf,
}

impl AppPaths {
    pub fn new() -> Self {
        let base = dirs_next().unwrap_or_else(|| PathBuf::from("."));
        Self { base }
    }

    pub fn base_dir(&self) -> &PathBuf {
        &self.base
    }
    pub fn db_path(&self) -> PathBuf {
        self.base.join("devhost.db")
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
    pub fn log_path(&self) -> PathBuf {
        self.base.join("devhost.log")
    }

    /// Ensure all required directories exist and seed static nginx config files.
    pub fn ensure_dirs(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.cert_dir())?;
        std::fs::create_dir_all(self.nginx_dir())?;
        std::fs::create_dir_all(self.nginx_logs())?;
        std::fs::create_dir_all(self.nginx_conf_subdir())?;

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

fn dirs_next() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(|p| PathBuf::from(p).join("DevHost"))
}
