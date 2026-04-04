use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Check if the DevHost CA is already installed in Windows LocalMachine\Root store
/// bằng cách so Thumbprint của file cert với store — tránh false positive khi dùng Subject match.
pub fn is_ca_installed(ca_cert_path: &Path) -> bool {
    let path_str = match ca_cert_path.to_str() {
        Some(s) => s,
        None => return false,
    };

    let script = format!(
        r#"
        try {{
            $cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 '{}'
            $thumb = $cert.Thumbprint
            
            $store = New-Object System.Security.Cryptography.X509Certificates.X509Store 'Root', 'LocalMachine'
            $store.Open('ReadOnly')
            
            $found = $false
            foreach ($c in $store.Certificates) {{
                if ($c.Thumbprint -eq $thumb) {{
                    $found = $true
                    break
                }}
            }}
            
            $store.Close()
            
            if ($found) {{ exit 0 }} else {{ exit 1 }}
        }} catch {{
            exit 1
        }}
        "#,
        path_str.replace('\'', "''")
    );

    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &script])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Install CA cert vào Windows LocalMachine\Root store.
///
/// Flow:
///   1. Ghi install script ra temp file (tránh ArgumentList parsing hell).
///   2. Ghi wrapper script gọi install script với elevated privileges.
///   3. Elevated script ghi kết quả OK/ERROR ra result file.
///   4. Wrapper chờ elevated process xong (-Wait), outer process đọc result file.
///   5. Cleanup tất cả temp files.
pub fn install_ca(ca_cert_path: &Path) -> anyhow::Result<()> {
    let cert_path_str = ca_cert_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid CA cert path (non-UTF8)"))?;

    // Dùng timestamp để tránh collision khi chạy concurrent
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let tmp_dir = std::env::temp_dir();
    let install_script_path = tmp_dir.join(format!("devhost_ca_install_{}.ps1", ts));
    let result_file_path = tmp_dir.join(format!("devhost_ca_result_{}.txt", ts));
    let wrapper_script_path = tmp_dir.join(format!("devhost_ca_wrapper_{}.ps1", ts));

    // Escape single quotes cho PowerShell string literals
    let cert_path_escaped = cert_path_str.replace('\'', "''");
    let result_path_escaped = result_file_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?
        .replace('\'', "''");
    let install_script_path_escaped = install_script_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?
        .replace('\'', "''");

    // Script chạy với elevated privileges: import cert, ghi kết quả ra file
    let install_script = format!(
        r#"
        try {{
            $cert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 '{cert}'
            $store = New-Object System.Security.Cryptography.X509Certificates.X509Store 'Root', 'LocalMachine'
            $store.Open('ReadWrite')
            $store.Add($cert)
            $store.Close()
            Set-Content -Path '{result}' -Value 'OK' -Encoding UTF8
        }} catch {{
            Set-Content -Path '{result}' -Value "ERROR: $_" -Encoding UTF8
        }}
        "#,
        cert = cert_path_escaped,
        result = result_path_escaped,
    );

    // Wrapper script: dùng Start-Process với -ArgumentList array để tránh parsing hell
    // -Wait đảm bảo wrapper block cho đến khi elevated child xong
    let wrapper_script = format!(
        r#"
        Start-Process powershell -Verb RunAs -Wait -ArgumentList @(
            '-NoProfile',
            '-NonInteractive',
            '-ExecutionPolicy', 'Bypass',
            '-WindowStyle', 'Hidden',
            '-File', '{install_script}'
        )
        "#,
        install_script = install_script_path_escaped,
    );

    std::fs::write(&install_script_path, &install_script)
        .map_err(|e| anyhow::anyhow!("Failed to write install script: {}", e))?;

    std::fs::write(&wrapper_script_path, &wrapper_script)
        .map_err(|e| anyhow::anyhow!("Failed to write wrapper script: {}", e))?;

    // Chạy wrapper (không cần elevated — nó tự spawn elevated child)
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
            wrapper_script_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid wrapper path"))?,
        ])
        .output();

    // Cleanup temp scripts — result file đọc xong mới xóa bên dưới
    let _ = std::fs::remove_file(&install_script_path);
    let _ = std::fs::remove_file(&wrapper_script_path);

    let output = output.map_err(|e| anyhow::anyhow!("Failed to spawn PowerShell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "PowerShell wrapper failed (exit {:?}): {}",
            output.status.code(),
            stderr
        );
    }

    // Đọc kết quả từ elevated script
    // Nếu user từ chối UAC, result file sẽ không tồn tại
    let result = std::fs::read_to_string(&result_file_path).unwrap_or_default();
    let _ = std::fs::remove_file(&result_file_path);

    let result = result.trim_start_matches('\u{feff}').trim();

    if result.is_empty() {
        anyhow::bail!(
            "CA install failed: no result written — user may have denied UAC prompt, \
             or elevated process crashed before writing"
        );
    }

    if !result.starts_with("OK") {
        anyhow::bail!("CA install failed: {}", result);
    }

    tracing::info!(
        path = %cert_path_str,
        "CA certificate installed to Windows LocalMachine\\Root store"
    );
    Ok(())
}
