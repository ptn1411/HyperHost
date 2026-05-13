use std::process::Command;

#[cfg(target_os = "windows")]
const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";


#[cfg(not(target_os = "windows"))]
const HOSTS_PATH: &str = "/etc/hosts";

const MARKER_START: &str = "# ── HyperHost BEGIN ──";
const MARKER_END: &str = "# ── HyperHost END ──";
// Backward-compat: also strip old DevHost markers from existing hosts files
const MARKER_START_LEGACY: &str = "# ── DevHost BEGIN ──";
const MARKER_END_LEGACY: &str = "# ── DevHost END ──";

/// Sync the list of domains into the system hosts file.
/// Only touches the HyperHost-managed block; leaves everything else untouched.
pub fn sync_hosts(domains: &[String]) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(HOSTS_PATH).unwrap_or_default();
    let cleaned = strip_managed_block(&content);

    let mut block = format!("\n{}\n", MARKER_START);
    for domain in domains {
        block.push_str(&format!("127.0.0.1\t{}\n", domain));
        // Add www prefix for top-level domains
        if domain.split('.').count() == 2 {
            block.push_str(&format!("127.0.0.1\twww.{}\n", domain));
        }
    }
    block.push_str(MARKER_END);
    block.push('\n');

    let final_content = format!("{}{}", cleaned.trim_end(), block);

    write_hosts(&final_content)?;
    flush_dns_cache();

    tracing::info!("Synced {} domains to hosts file", domains.len());
    Ok(())
}

/// Remove all domains managed by HyperHost from hosts file.
pub fn remove_all() -> anyhow::Result<()> {
    let content = std::fs::read_to_string(HOSTS_PATH).unwrap_or_default();
    let cleaned = strip_managed_block(&content);
    write_hosts(cleaned.trim_end())?;
    flush_dns_cache();
    tracing::info!("Removed HyperHost block from hosts file");
    Ok(())
}

/// Write content to the hosts file, using elevated privileges if needed.
fn write_hosts(content: &str) -> anyhow::Result<()> {
    let tmp = format!("{}.__hyperhost_tmp", HOSTS_PATH);
    if std::fs::write(&tmp, content).is_ok() {
        if std::fs::rename(&tmp, HOSTS_PATH).is_ok() {
            return Ok(());
        }
        let _ = std::fs::remove_file(&tmp);
    }

    crate::elevation::write_file_elevated(HOSTS_PATH, content)
}

fn flush_dns_cache() {
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("ipconfig").args(["/flushdns"]).output();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("dscacheutil").arg("-flushcache").output();
        let _ = Command::new("killall")
            .args(["-HUP", "mDNSResponder"])
            .output();
    }
    #[cfg(target_os = "linux")]
    {
        if Command::new("systemctl")
            .args(["is-active", "--quiet", "systemd-resolved"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            let _ = Command::new("resolvectl").arg("flush-caches").output();
        } else {
            let _ = Command::new("nscd").args(["-i", "hosts"]).output();
        }
    }
}


/// Strip both current (HyperHost) and legacy (DevHost) managed blocks.
fn strip_managed_block(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_block = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == MARKER_START || trimmed == MARKER_START_LEGACY {
            in_block = true;
            continue;
        }
        if trimmed == MARKER_END || trimmed == MARKER_END_LEGACY {
            in_block = false;
            continue;
        }
        if !in_block {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}
