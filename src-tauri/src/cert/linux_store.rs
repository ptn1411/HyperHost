use std::path::Path;
use std::process::Command;

const CA_DEST: &str = "/usr/local/share/ca-certificates/hyperhost-ca.crt";

/// Check if the HyperHost CA is already installed.
pub fn is_ca_installed(_ca_cert_path: &Path) -> bool {
    std::path::Path::new(CA_DEST).exists()
}

/// Install CA cert into the system trust store.
/// Requires admin privileges (sudo).
pub fn install_ca(ca_cert_path: &Path) -> anyhow::Result<()> {
    let cert_path = ca_cert_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid CA cert path"))?;

    // Copy cert to system ca-certificates directory
    let copy_status = Command::new("sudo")
        .args(["cp", cert_path, CA_DEST])
        .status()?;

    if !copy_status.success() {
        anyhow::bail!("Failed to copy CA cert to {} — ensure you have sudo privileges", CA_DEST);
    }

    // Update system trust store
    let update_status = Command::new("sudo")
        .args(["update-ca-certificates"])
        .status();

    // Fallback for non-Debian distros (Fedora/Arch)
    if update_status.map(|s| !s.success()).unwrap_or(true) {
        let _ = Command::new("sudo")
            .args(["trust", "anchor", "--store", CA_DEST])
            .status();
    }

    tracing::info!("CA certificate installed to Linux system trust store");
    Ok(())
}
