use std::path::Path;
use std::process::Command;

/// Check if the HyperHost CA is already trusted in macOS System Keychain.
pub fn is_ca_installed(ca_cert_path: &Path) -> bool {
    let output = Command::new("security")
        .args([
            "verify-cert",
            "-c",
            ca_cert_path.to_str().unwrap_or_default(),
            "-p",
            "ssl",
        ])
        .output();
    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Install CA cert into macOS System Keychain and mark as trusted.
/// Requires admin privileges (will prompt via sudo/osascript).
pub fn install_ca(ca_cert_path: &Path) -> anyhow::Result<()> {
    let cert_path = ca_cert_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid CA cert path"))?;

    // Add to System Keychain
    let status = Command::new("sudo")
        .args([
            "security",
            "add-trusted-cert",
            "-d",
            "-r",
            "trustRoot",
            "-k",
            "/Library/Keychains/System.keychain",
            cert_path,
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("security add-trusted-cert failed — ensure you have admin privileges");
    }

    tracing::info!("CA certificate installed to macOS System Keychain");
    Ok(())
}
