use std::process::Command;

/// Check if the current process has administrative privileges.
#[cfg(windows)]
pub fn is_admin() -> bool {
    #[link(name = "shell32")]
    extern "system" {
        fn IsUserAnAdmin() -> i32;
    }
    unsafe { IsUserAnAdmin() != 0 }
}

#[cfg(unix)]
pub fn is_admin() -> bool {
    extern "C" {
        fn geteuid() -> u32;
    }
    unsafe { geteuid() == 0 }
}

/// Write content to a protected system file using elevated privileges.
///
/// Windows: triggers a UAC prompt via PowerShell `Start-Process -Verb RunAs`.
/// Unix: uses `sudo tee` (requires terminal for password prompt).
#[cfg(windows)]
pub fn write_file_elevated(target: &str, content: &str) -> anyhow::Result<()> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let tmp_dir = std::env::temp_dir();
    let content_file = tmp_dir.join(format!("hyperhost_elev_content_{}.tmp", ts));
    let result_file = tmp_dir.join(format!("hyperhost_elev_result_{}.txt", ts));
    let install_script = tmp_dir.join(format!("hyperhost_elev_install_{}.ps1", ts));
    let wrapper_script = tmp_dir.join(format!("hyperhost_elev_wrapper_{}.ps1", ts));

    std::fs::write(&content_file, content)
        .map_err(|e| anyhow::anyhow!("Failed to write temp content: {}", e))?;

    let content_path_esc = content_file
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?
        .replace('\'', "''");
    let target_esc = target.replace('\'', "''");
    let result_path_esc = result_file
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid result path"))?
        .replace('\'', "''");
    let install_path_esc = install_script
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid install script path"))?
        .replace('\'', "''");

    let install_ps1 = format!(
        r#"
        try {{
            Copy-Item -Path '{content}' -Destination '{target}' -Force
            Set-Content -Path '{result}' -Value 'OK' -Encoding UTF8
        }} catch {{
            Set-Content -Path '{result}' -Value "ERROR: $_" -Encoding UTF8
        }}
        "#,
        content = content_path_esc,
        target = target_esc,
        result = result_path_esc,
    );

    let wrapper_ps1 = format!(
        r#"
        Start-Process powershell -Verb RunAs -Wait -ArgumentList @(
            '-NoProfile',
            '-NonInteractive',
            '-ExecutionPolicy', 'Bypass',
            '-WindowStyle', 'Hidden',
            '-File', '{install_script}'
        )
        "#,
        install_script = install_path_esc,
    );

    std::fs::write(&install_script, &install_ps1)
        .map_err(|e| anyhow::anyhow!("Failed to write install script: {}", e))?;
    std::fs::write(&wrapper_script, &wrapper_ps1)
        .map_err(|e| anyhow::anyhow!("Failed to write wrapper script: {}", e))?;

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
            wrapper_script
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid wrapper path"))?,
        ])
        .output();

    // Cleanup temp files
    let _ = std::fs::remove_file(&install_script);
    let _ = std::fs::remove_file(&wrapper_script);
    let _ = std::fs::remove_file(&content_file);

    let output = output.map_err(|e| anyhow::anyhow!("Failed to spawn PowerShell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Elevation wrapper failed (exit {:?}): {}",
            output.status.code(),
            stderr
        );
    }

    let result = std::fs::read_to_string(&result_file).unwrap_or_default();
    let _ = std::fs::remove_file(&result_file);

    let result = result.trim_start_matches('\u{feff}').trim();

    if result.is_empty() {
        anyhow::bail!(
            "Elevated write failed: no result — user may have denied UAC prompt"
        );
    }

    if !result.starts_with("OK") {
        anyhow::bail!("Elevated write failed: {}", result);
    }

    tracing::info!(target_path = %target, "File written via elevated process");
    Ok(())
}

#[cfg(unix)]
pub fn write_file_elevated(target: &str, content: &str) -> anyhow::Result<()> {
    use std::io::Write;

    let mut child = Command::new("sudo")
        .args(["tee", target])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to run sudo tee: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!("sudo tee failed — insufficient privileges to write {}", target);
    }

    tracing::info!(target_path = %target, "File written via sudo");
    Ok(())
}
