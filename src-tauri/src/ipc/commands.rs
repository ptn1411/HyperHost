use crate::db::{DomainConfig, DomainStatus};
use crate::state::AppState;

#[derive(Debug, serde::Serialize)]
pub struct CaStatus {
    pub installed: bool,
    pub fingerprint: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct NginxInfo {
    pub running: bool,
}

#[tauri::command]
pub async fn add_domain(
    domain: String,
    upstream: String,
    advanced_config: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<DomainStatus, String> {
    // Validate
    if !domain.ends_with(".test") && !domain.ends_with(".local") {
        return Err("Domain must end with .test or .local".into());
    }
    if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
        return Err("Upstream must start with http:// or https://".into());
    }

    // 1. Issue cert — try rcgen first, fallback to mkcert
    let cert_dir = state.paths.cert_dir();
    std::fs::create_dir_all(&cert_dir).map_err(|e| e.to_string())?;

    let (cert_pem, key_pem) = match state.ca.issue_for_domain(&domain) {
        Ok((cert, key)) => {
            tracing::info!("rcgen: cert issued for {}", &domain);
            // Write cert files
            std::fs::write(cert_dir.join(format!("{}.crt", &domain)), &cert)
                .map_err(|e| e.to_string())?;
            std::fs::write(cert_dir.join(format!("{}.key", &domain)), &key)
                .map_err(|e| e.to_string())?;
            (cert, key)
        }
        Err(rcgen_err) => {
            tracing::warn!(
                "rcgen failed for {}: {}, trying mkcert fallback...",
                &domain,
                rcgen_err
            );

            // Fallback to mkcert binary
            let mkcert = crate::cert::mkcert::MkcertRunner::find().ok_or_else(|| {
                format!("rcgen failed ({}) and mkcert binary not found", rcgen_err)
            })?;

            mkcert.issue_for_domain(&domain, &cert_dir).map_err(|e| {
                format!(
                    "Both rcgen and mkcert failed. rcgen: {}. mkcert: {}",
                    rcgen_err, e
                )
            })?;

            // Read back the generated files
            let cert = std::fs::read_to_string(cert_dir.join(format!("{}.crt", &domain)))
                .map_err(|e| e.to_string())?;
            let key = std::fs::read_to_string(cert_dir.join(format!("{}.key", &domain)))
                .map_err(|e| e.to_string())?;
            (cert, key)
        }
    };

    // 3. Save to DB
    let expiry = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(crate::cert::ca::CERT_VALIDITY_DAYS))
        .map(|d| d.to_rfc3339());

    let cfg = DomainConfig {
        id: None,
        domain: domain.clone(),
        upstream,
        enabled: true,
        cors_enabled: false,
        cert_expiry: expiry.clone(),
        created_at: None,
        advanced_config,
    };
    state
        .db
        .upsert_domain(&cfg, &cert_pem, &key_pem)
        .map_err(|e| e.to_string())?;

    // 4. Sync hosts file
    let active = state.db.list_enabled_domains().map_err(|e| e.to_string())?;
    crate::dns::hosts::sync_hosts(&active).map_err(|e| e.to_string())?;

    // 5. Regenerate nginx config + reload
    rebuild_nginx(&state).map_err(|e| e.to_string())?;

    Ok(DomainStatus {
        config: cfg,
        cert_valid: true,
        cert_expiry: expiry,
    })
}

#[tauri::command]
pub async fn edit_domain(
    old_domain: String,
    domain: String,
    upstream: String,
    advanced_config: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<DomainStatus, String> {
    // Validate new values
    if !domain.ends_with(".test") && !domain.ends_with(".local") {
        return Err("Domain must end with .test or .local".into());
    }
    if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
        return Err("Upstream must start with http:// or https://".into());
    }

    // If domain name changed, remove the old one first
    if old_domain != domain {
        state.db.remove_domain(&old_domain).map_err(|e| e.to_string())?;
        // Remove old cert files
        let cert_dir = state.paths.cert_dir();
        let _ = std::fs::remove_file(cert_dir.join(format!("{}.crt", &old_domain)));
        let _ = std::fs::remove_file(cert_dir.join(format!("{}.key", &old_domain)));
    }

    // Issue cert for the (possibly new) domain
    let cert_dir = state.paths.cert_dir();
    std::fs::create_dir_all(&cert_dir).map_err(|e| e.to_string())?;

    let (cert_pem, key_pem) = match state.ca.issue_for_domain(&domain) {
        Ok((cert, key)) => {
            tracing::info!("rcgen: cert issued for {}", &domain);
            std::fs::write(cert_dir.join(format!("{}.crt", &domain)), &cert)
                .map_err(|e| e.to_string())?;
            std::fs::write(cert_dir.join(format!("{}.key", &domain)), &key)
                .map_err(|e| e.to_string())?;
            (cert, key)
        }
        Err(rcgen_err) => {
            tracing::warn!("rcgen failed for {}: {}, trying mkcert fallback...", &domain, rcgen_err);
            let mkcert = crate::cert::mkcert::MkcertRunner::find().ok_or_else(|| {
                format!("rcgen failed ({}) and mkcert binary not found", rcgen_err)
            })?;
            mkcert.issue_for_domain(&domain, &cert_dir).map_err(|e| {
                format!("Both rcgen and mkcert failed. rcgen: {}. mkcert: {}", rcgen_err, e)
            })?;
            let cert = std::fs::read_to_string(cert_dir.join(format!("{}.crt", &domain)))
                .map_err(|e| e.to_string())?;
            let key = std::fs::read_to_string(cert_dir.join(format!("{}.key", &domain)))
                .map_err(|e| e.to_string())?;
            (cert, key)
        }
    };

    let expiry = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(crate::cert::ca::CERT_VALIDITY_DAYS))
        .map(|d| d.to_rfc3339());

    // Preserve cors_enabled from existing domain if renaming
    let cors_enabled = if old_domain != domain {
        false
    } else {
        state.db.list_domains().ok()
            .and_then(|ds| ds.into_iter().find(|d| d.domain == domain))
            .map(|d| d.cors_enabled)
            .unwrap_or(false)
    };

    let cfg = DomainConfig {
        id: None,
        domain: domain.clone(),
        upstream,
        enabled: true,
        cors_enabled,
        cert_expiry: expiry.clone(),
        created_at: None,
        advanced_config,
    };
    state.db.upsert_domain(&cfg, &cert_pem, &key_pem).map_err(|e| e.to_string())?;

    // Sync hosts & rebuild nginx
    let active = state.db.list_enabled_domains().map_err(|e| e.to_string())?;
    crate::dns::hosts::sync_hosts(&active).map_err(|e| e.to_string())?;
    rebuild_nginx(&state).map_err(|e| e.to_string())?;

    Ok(DomainStatus {
        config: cfg,
        cert_valid: true,
        cert_expiry: expiry,
    })
}

#[tauri::command]
pub async fn list_domains(state: tauri::State<'_, AppState>) -> Result<Vec<DomainStatus>, String> {
    let domains = state.db.list_domains().map_err(|e| e.to_string())?;
    let result = domains
        .into_iter()
        .map(|cfg| {
            let cert_valid = cfg.cert_expiry.as_ref().map_or(false, |exp| {
                chrono::DateTime::parse_from_rfc3339(exp)
                    .map(|d| d > chrono::Utc::now())
                    .unwrap_or(false)
            });
            DomainStatus {
                cert_expiry: cfg.cert_expiry.clone(),
                cert_valid,
                config: cfg,
            }
        })
        .collect();
    Ok(result)
}

#[tauri::command]
pub async fn remove_domain(
    domain: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Remove from DB
    state.db.remove_domain(&domain).map_err(|e| e.to_string())?;

    // Remove cert files
    let cert_dir = state.paths.cert_dir();
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.crt", &domain)));
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.key", &domain)));

    // Sync hosts
    let active = state.db.list_enabled_domains().map_err(|e| e.to_string())?;
    crate::dns::hosts::sync_hosts(&active).map_err(|e| e.to_string())?;

    // Rebuild nginx
    rebuild_nginx(&state).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_domain(
    domain: String,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let new_state = state.db.toggle_domain(&domain).map_err(|e| e.to_string())?;

    let active = state.db.list_enabled_domains().map_err(|e| e.to_string())?;
    crate::dns::hosts::sync_hosts(&active).map_err(|e| e.to_string())?;
    rebuild_nginx(&state).map_err(|e| e.to_string())?;

    Ok(new_state)
}

#[tauri::command]
pub async fn install_ca(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let ca_cert = state.paths.ca_cert();

    // Platform-specific system trust store installation
    let result = {
        #[cfg(target_os = "windows")]
        {
            crate::cert::windows_store::install_ca(&ca_cert)
        }
        #[cfg(target_os = "macos")]
        {
            crate::cert::macos_store::install_ca(&ca_cert)
        }
        #[cfg(target_os = "linux")]
        {
            crate::cert::linux_store::install_ca(&ca_cert)
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(anyhow::anyhow!("CA installation not supported on this platform"))
        }
    };

    if let Err(ref e) = result {
        tracing::warn!("System CA install failed: {}", e);
    }

    // Also try mkcert -install for Firefox NSS store coverage
    if let Some(mkcert) = crate::cert::mkcert::MkcertRunner::find() {
        if let Err(e) = mkcert.install_ca() {
            tracing::warn!("mkcert -install failed: {}", e);
        } else {
            tracing::info!("mkcert -install succeeded (Firefox NSS store covered)");
        }
    }

    result.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn ca_status(state: tauri::State<'_, AppState>) -> Result<CaStatus, String> {
    let ca_cert = state.paths.ca_cert();
    let installed = {
        #[cfg(target_os = "windows")]
        { crate::cert::windows_store::is_ca_installed(&ca_cert) }
        #[cfg(target_os = "macos")]
        { crate::cert::macos_store::is_ca_installed(&ca_cert) }
        #[cfg(target_os = "linux")]
        { crate::cert::linux_store::is_ca_installed(&ca_cert) }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        { false }
    };
    Ok(CaStatus {
        installed,
        fingerprint: state.ca.fingerprint(),
    })
}

#[tauri::command]
pub async fn nginx_status(state: tauri::State<'_, AppState>) -> Result<NginxInfo, String> {
    Ok(NginxInfo {
        running: state.nginx.is_running(),
    })
}

#[tauri::command]
pub async fn nginx_start(state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Check if nginx binary exists before trying to start
    if !state.nginx.exe.exists() {
        #[cfg(target_os = "macos")]
        return Err("nginx not found. Install it with: brew install nginx".into());
        #[cfg(target_os = "linux")]
        return Err("nginx not found. Install it with: sudo apt install nginx".into());
        #[cfg(target_os = "windows")]
        return Err(format!("nginx binary not found at: {}", state.nginx.exe.display()));
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        return Err("nginx binary not found".into());
    }

    // Rebuild config before starting to ensure it's up to date
    rebuild_nginx(&state).map_err(|e| e.to_string())?;
    state.nginx.start().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn nginx_stop(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.nginx.stop().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_nginx_log(
    lines: usize,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let log_path = state.paths.nginx_logs().join("error.log");
    let content = std::fs::read_to_string(&log_path).unwrap_or_default();
    let all_lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let start = all_lines.len().saturating_sub(lines);
    Ok(all_lines[start..].to_vec())
}

#[tauri::command]
pub async fn start_tunnel(
    domain: String,
    state: tauri::State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    if let Ok(domains) = state.db.list_domains() {
        if let Some(config) = domains.iter().find(|d| d.domain == domain) {
            return state.cloudflared.start(app_handle, domain, config.upstream.clone()).map_err(|e| e.to_string());
        }
    }
    Err("Domain not found in config".to_string())
}

#[tauri::command]
pub async fn stop_tunnel(
    domain: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.cloudflared.stop(&domain);
    Ok(())
}

// ── Named Tunnel commands ──

#[tauri::command]
pub async fn cloudflare_login(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let exe = crate::resolve_cloudflared_exe();
    crate::cloudflare::named_tunnel::NamedTunnelManager::login(&exe).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn cloudflare_login_status() -> Result<bool, String> {
    Ok(crate::cloudflare::named_tunnel::NamedTunnelManager::is_logged_in())
}

#[tauri::command]
pub async fn list_named_tunnels(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<crate::db::NamedTunnelConfig>, String> {
    state.db.list_named_tunnels().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_named_tunnel(
    tunnel_name: String,
    hostname: String,
    upstream: String,
    state: tauri::State<'_, AppState>,
) -> Result<crate::db::NamedTunnelConfig, String> {
    if tunnel_name.trim().is_empty() {
        return Err("Tunnel name cannot be empty".into());
    }
    if hostname.trim().is_empty() {
        return Err("Hostname cannot be empty".into());
    }
    if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
        return Err("Upstream must start with http:// or https://".into());
    }

    let cfg = crate::db::NamedTunnelConfig {
        id: None,
        tunnel_name: tunnel_name.clone(),
        tunnel_id: None,
        credentials_path: None,
        hostname,
        upstream,
        enabled: true,
        created_at: None,
    };
    state.db.insert_named_tunnel(&cfg).map_err(|e| e.to_string())?;
    state
        .db
        .get_named_tunnel(&tunnel_name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Tunnel not found after insert".into())
}

#[tauri::command]
pub async fn provision_named_tunnel(
    tunnel_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let exe = crate::resolve_cloudflared_exe();
    let (tunnel_id, creds_path) =
        crate::cloudflare::named_tunnel::NamedTunnelManager::create_tunnel(&exe, &tunnel_name)
            .map_err(|e| e.to_string())?;

    state
        .db
        .update_named_tunnel_credentials(&tunnel_name, &tunnel_id, &creds_path)
        .map_err(|e| e.to_string())?;

    // Generate config.yml right away
    let cfg = state
        .db
        .get_named_tunnel(&tunnel_name)
        .map_err(|e| e.to_string())?
        .ok_or("Tunnel not found")?;

    let config_path = state.paths.tunnel_config(&tunnel_name);
    crate::cloudflare::named_tunnel::NamedTunnelManager::generate_config(
        &config_path,
        &tunnel_id,
        &creds_path,
        &[(cfg.hostname, cfg.upstream)],
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_named_tunnel(
    tunnel_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let cfg = state
        .db
        .get_named_tunnel(&tunnel_name)
        .map_err(|e| e.to_string())?
        .ok_or("Tunnel not found")?;

    let tunnel_id = cfg
        .tunnel_id
        .as_deref()
        .ok_or("Tunnel not provisioned yet — click Provision first")?;
    let creds = cfg
        .credentials_path
        .as_deref()
        .ok_or("Credentials path missing — re-provision the tunnel")?;

    let config_path = state.paths.tunnel_config(&tunnel_name);

    // Re-generate config in case hostname/upstream changed
    crate::cloudflare::named_tunnel::NamedTunnelManager::generate_config(
        &config_path,
        tunnel_id,
        creds,
        &[(cfg.hostname, cfg.upstream)],
    )
    .map_err(|e| e.to_string())?;

    let exe = crate::resolve_cloudflared_exe();
    state
        .named_tunnels
        .start(&tunnel_name, &config_path, &exe)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_named_tunnel(
    tunnel_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.named_tunnels.stop(&tunnel_name);
    Ok(())
}

#[tauri::command]
pub async fn named_tunnel_running(
    tunnel_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    Ok(state.named_tunnels.is_running(&tunnel_name))
}

#[tauri::command]
pub async fn remove_named_tunnel(
    tunnel_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.named_tunnels.stop(&tunnel_name);
    // Remove config.yml
    let config_path = state.paths.tunnel_config(&tunnel_name);
    let _ = std::fs::remove_file(config_path);
    state
        .db
        .remove_named_tunnel(&tunnel_name)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn toggle_cors(
    domain: String,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    let new_state = state.db.toggle_cors(&domain).map_err(|e| e.to_string())?;
    rebuild_nginx(&state).map_err(|e| e.to_string())?;
    Ok(new_state)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ExportData {
    version: u32,
    domains: Vec<crate::db::DomainConfig>,
}

#[tauri::command]
pub async fn export_domains(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let domains = state.db.list_domains().map_err(|e| e.to_string())?;
    // Strip sensitive/internal fields before export
    let clean: Vec<_> = domains
        .into_iter()
        .map(|d| crate::db::DomainConfig {
            id: None,
            created_at: None,
            cert_expiry: None,
            ..d
        })
        .collect();
    let data = ExportData { version: 1, domains: clean };
    serde_json::to_string_pretty(&data).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn import_domains(
    json: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DomainStatus>, String> {
    let data: ExportData = serde_json::from_str(&json)
        .map_err(|e| format!("Invalid import file: {}", e))?;

    let mut imported = Vec::new();
    let cert_dir = state.paths.cert_dir();
    std::fs::create_dir_all(&cert_dir).map_err(|e| e.to_string())?;

    for cfg in data.domains {
        // Validate
        if !cfg.domain.ends_with(".test") && !cfg.domain.ends_with(".local") {
            continue; // skip invalid domains silently
        }
        if !cfg.upstream.starts_with("http://") && !cfg.upstream.starts_with("https://") {
            continue;
        }

        let (cert_pem, key_pem) = match state.ca.issue_for_domain(&cfg.domain) {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!("import: failed to issue cert for {}: {}", cfg.domain, e);
                continue;
            }
        };

        let _ = std::fs::write(cert_dir.join(format!("{}.crt", cfg.domain)), &cert_pem);
        let _ = std::fs::write(cert_dir.join(format!("{}.key", cfg.domain)), &key_pem);

        let expiry = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(crate::cert::ca::CERT_VALIDITY_DAYS))
            .map(|d| d.to_rfc3339());

        let new_cfg = crate::db::DomainConfig {
            id: None,
            cert_expiry: expiry.clone(),
            created_at: None,
            ..cfg
        };

        if let Err(e) = state.db.upsert_domain(&new_cfg, &cert_pem, &key_pem) {
            tracing::warn!("import: DB upsert failed for {}: {}", new_cfg.domain, e);
            continue;
        }

        imported.push(DomainStatus {
            cert_valid: true,
            cert_expiry: expiry,
            config: new_cfg,
        });
    }

    // Sync hosts + rebuild nginx once after all imports
    if !imported.is_empty() {
        let active = state.db.list_enabled_domains().map_err(|e| e.to_string())?;
        crate::dns::hosts::sync_hosts(&active).map_err(|e| e.to_string())?;
        rebuild_nginx(&state).map_err(|e| e.to_string())?;
    }

    Ok(imported)
}

fn rebuild_nginx(state: &AppState) -> anyhow::Result<()> {
    let all = state.db.list_domains()?;
    let nginx_conf = crate::nginx::config::generate(
        &all,
        state.paths.cert_dir().to_str().unwrap(),
        state.paths.nginx_dir().to_str().unwrap(),
    );
    std::fs::write(&state.paths.nginx_conf(), nginx_conf)?;

    if state.nginx.is_running() {
        state.nginx.reload()?;
    }
    Ok(())
}

// ── App Settings commands ──

#[derive(serde::Serialize)]
pub struct AppSettings {
    pub autostart: bool,
    pub minimize_to_tray: bool,
}

#[tauri::command]
pub async fn get_app_settings(state: tauri::State<'_, AppState>) -> Result<AppSettings, String> {
    let autostart = {
        #[cfg(target_os = "windows")]
        { is_autostart_windows() }
        #[cfg(not(target_os = "windows"))]
        { false }
    };
    let minimize_to_tray = state
        .minimize_to_tray
        .load(std::sync::atomic::Ordering::Relaxed);
    Ok(AppSettings {
        autostart,
        minimize_to_tray,
    })
}

#[tauri::command]
pub async fn set_autostart(
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .db
        .set_setting("autostart", if enabled { "true" } else { "false" })
        .map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    toggle_autostart_windows(enabled).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn set_minimize_to_tray(
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .minimize_to_tray
        .store(enabled, std::sync::atomic::Ordering::Relaxed);
    state
        .db
        .set_setting("minimize_to_tray", if enabled { "true" } else { "false" })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn toggle_autostart_windows(enabled: bool) -> anyhow::Result<()> {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_SET_VALUE,
    )?;
    if enabled {
        let exe = std::env::current_exe()?;
        run.set_value("HyperHost", &exe.to_string_lossy().to_string())?;
    } else {
        let _ = run.delete_value("HyperHost");
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn is_autostart_windows() -> bool {
    use winreg::enums::{HKEY_CURRENT_USER, KEY_READ};
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run) = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run",
        KEY_READ,
    ) {
        run.get_value::<String, _>("HyperHost").is_ok()
    } else {
        false
    }
}
