use std::path::{Path, PathBuf};
use std::process::Command;

/// Wrapper around the mkcert binary for fallback cert generation.
pub struct MkcertRunner {
    exe: PathBuf,
}

impl MkcertRunner {
    /// Locate the mkcert binary. Checks sidecar locations first, then PATH.
    pub fn find() -> Option<Self> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_default();

        let sidecar = crate::sidecar_name("mkcert");
        let fallback = format!("mkcert{}", std::env::consts::EXE_SUFFIX);
        let candidates = [
            exe_dir.join(&sidecar),
            exe_dir.join(&fallback),
            PathBuf::from("src-tauri/binaries").join(&sidecar),
            PathBuf::from("binaries").join(&sidecar),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                tracing::info!("Found mkcert at: {}", candidate.display());
                return Some(Self {
                    exe: candidate.clone(),
                });
            }
        }

        // Try PATH
        if Command::new("mkcert").arg("-version").output().is_ok() {
            tracing::info!("Found mkcert in PATH");
            return Some(Self {
                exe: PathBuf::from("mkcert"),
            });
        }

        tracing::warn!("mkcert binary not found");
        None
    }

    /// Install the mkcert root CA into system and browser trust stores.
    /// This handles Firefox NSS store automatically.
    pub fn install_ca(&self) -> anyhow::Result<()> {
        let output = Command::new(&self.exe).arg("-install").output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("mkcert -install failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("mkcert -install: {}", stdout.trim());
        Ok(())
    }

    /// Install CA using a custom CAROOT directory (use DevHost's own CA).
    pub fn install_ca_with_root(&self, ca_dir: &Path) -> anyhow::Result<()> {
        let output = Command::new(&self.exe)
            .env("CAROOT", ca_dir.to_str().unwrap_or_default())
            .arg("-install")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("mkcert -install (custom CAROOT) failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::info!("mkcert -install (custom CAROOT): {}", stdout.trim());
        Ok(())
    }

    /// Generate a certificate for one or more domains.
    /// Writes cert and key files to `output_dir`.
    /// Returns `(cert_path, key_path)`.
    pub fn issue_for_domain(
        &self,
        domain: &str,
        output_dir: &Path,
    ) -> anyhow::Result<(PathBuf, PathBuf)> {
        std::fs::create_dir_all(output_dir)?;

        let cert_path = output_dir.join(format!("{}.crt", domain));
        let key_path = output_dir.join(format!("{}.key", domain));

        let output = Command::new(&self.exe)
            .args([
                "-cert-file",
                cert_path.to_str().unwrap(),
                "-key-file",
                key_path.to_str().unwrap(),
                domain,
                &format!("*.{}", domain),
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("mkcert cert generation failed for {}: {}", domain, stderr);
        }

        tracing::info!(
            "mkcert issued cert for {} → {}",
            domain,
            cert_path.display()
        );
        Ok((cert_path, key_path))
    }
}
