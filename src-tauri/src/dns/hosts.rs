use std::process::Command;

const HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";
const MARKER_START: &str = "# ── DevHost BEGIN ──";
const MARKER_END: &str = "# ── DevHost END ──";

/// Sync the list of domains into the Windows hosts file.
/// Only touches the DevHost-managed block; leaves everything else untouched.
pub fn sync_hosts(domains: &[String]) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(HOSTS_PATH).unwrap_or_default();
    let cleaned = strip_devhost_block(&content);

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

    // Atomic write: write to temp → rename
    let tmp = format!("{}.__devhost_tmp", HOSTS_PATH);
    std::fs::write(&tmp, &final_content)?;
    std::fs::rename(&tmp, HOSTS_PATH)?;

    // Flush DNS cache
    let _ = Command::new("ipconfig").args(["/flushdns"]).output();

    tracing::info!("Synced {} domains to hosts file", domains.len());
    Ok(())
}

/// Remove all domains managed by DevHost from hosts file.
pub fn remove_all() -> anyhow::Result<()> {
    let content = std::fs::read_to_string(HOSTS_PATH).unwrap_or_default();
    let cleaned = strip_devhost_block(&content);
    std::fs::write(HOSTS_PATH, cleaned.trim_end())?;
    let _ = Command::new("ipconfig").args(["/flushdns"]).output();
    tracing::info!("Removed DevHost block from hosts file");
    Ok(())
}

fn strip_devhost_block(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    let mut in_block = false;
    for line in content.lines() {
        if line.trim() == MARKER_START {
            in_block = true;
            continue;
        }
        if line.trim() == MARKER_END {
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
