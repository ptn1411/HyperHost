pub mod cert;
#[cfg(feature = "gui")]
pub mod cloudflare;
pub mod db;
pub mod detect;
pub mod dns;
#[cfg(feature = "gui")]
pub mod docker;
#[cfg(feature = "gui")]
pub mod ipc;
pub mod nginx;
pub mod paths;
pub mod state;

use state::AppState;

/// Initialize core app state (shared between GUI and CLI).
pub fn init_state() -> anyhow::Result<AppState> {
    let paths = paths::AppPaths::new();
    paths.ensure_dirs()?;

    let db = db::Database::open(&paths.db_path())?;
    let ca = cert::ca::LocalCA::load_or_create(paths.base_dir())?;

    // Auto-renew certs expiring within 30 days
    renew_expiring_certs(&db, &ca, &paths);

    let nginx_exe = resolve_nginx_exe();
    let nginx = nginx::NginxManager::new(nginx_exe, paths.nginx_conf(), paths.nginx_dir());

    Ok(AppState {
        paths,
        db,
        ca,
        nginx,
        #[cfg(feature = "gui")]
        cloudflared: cloudflare::CloudflaredManager::new(),
        #[cfg(feature = "gui")]
        named_tunnels: cloudflare::named_tunnel::NamedTunnelManager::new(),
    })
}

/// Re-issue certs for domains expiring within RENEW_THRESHOLD_DAYS.
fn renew_expiring_certs(
    db: &db::Database,
    ca: &cert::ca::LocalCA,
    paths: &paths::AppPaths,
) {
    const RENEW_THRESHOLD_DAYS: i64 = 30;

    let domains = match db.list_domains() {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("auto-renew: failed to list domains: {}", e);
            return;
        }
    };

    let threshold = chrono::Utc::now()
        + chrono::Duration::days(RENEW_THRESHOLD_DAYS);

    for cfg in domains {
        let expiring = cfg.cert_expiry.as_ref().map_or(true, |exp| {
            chrono::DateTime::parse_from_rfc3339(exp)
                .map(|d| d < threshold)
                .unwrap_or(true)
        });

        if !expiring {
            continue;
        }

        tracing::info!("auto-renew: cert for {} expiring soon, re-issuing", cfg.domain);

        let cert_dir = paths.cert_dir();
        let (cert_pem, key_pem) = match ca.issue_for_domain(&cfg.domain) {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!("auto-renew: failed to issue cert for {}: {}", cfg.domain, e);
                continue;
            }
        };

        // Write cert files
        let _ = std::fs::write(cert_dir.join(format!("{}.crt", cfg.domain)), &cert_pem);
        let _ = std::fs::write(cert_dir.join(format!("{}.key", cfg.domain)), &key_pem);

        // Update expiry in DB
        let new_expiry = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(cert::ca::CERT_VALIDITY_DAYS))
            .map(|d| d.to_rfc3339());

        let updated = db::DomainConfig {
            cert_expiry: new_expiry,
            ..cfg
        };
        if let Err(e) = db.upsert_domain(&updated, &cert_pem, &key_pem) {
            tracing::warn!("auto-renew: failed to update DB for {}: {}", updated.domain, e);
        } else {
            tracing::info!("auto-renew: cert renewed for {}", updated.domain);
        }
    }
}

/// Launch the Tauri GUI application.
#[cfg(feature = "gui")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();

    let app_state = init_state().expect("Failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .setup(|app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            use tauri::Manager;

            let state = app.state::<AppState>();
            if let Err(e) = state.nginx.start() {
                tracing::error!("Failed to auto-start Nginx: {}", e);
            }

            crate::nginx::tail::start_tailing(
                app.handle().clone(),
                state.paths.nginx_logs().join("access.json"),
            );

            // If launched with --minimized (e.g. autostart with "start hidden" option)
            if std::env::args().any(|a| a == "--minimized") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            // Create Tray Menu
            let show_i = MenuItem::with_id(app, "show", "Open HyperHost", true, None::<&str>)?;
            let hide_i = MenuItem::with_id(app, "hide", "Hide to Tray", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

            let app_handle_1 = app.handle().clone();
            let app_handle_2 = app.handle().clone();

            // Build Tray Icon
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .menu_on_left_click(false)
                .on_menu_event(move |_app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app_handle_1.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app_handle_1.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "quit" => {
                        app_handle_1.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        if let Some(window) = app_handle_2.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::add_domain,
            ipc::commands::edit_domain,
            ipc::commands::list_domains,
            ipc::commands::remove_domain,
            ipc::commands::toggle_domain,
            ipc::commands::install_ca,
            ipc::commands::ca_status,
            ipc::commands::nginx_status,
            ipc::commands::nginx_start,
            ipc::commands::nginx_stop,
            ipc::commands::get_nginx_log,
            ipc::commands::start_tunnel,
            ipc::commands::stop_tunnel,
            ipc::commands::cloudflare_login,
            ipc::commands::cloudflare_login_status,
            ipc::commands::list_named_tunnels,
            ipc::commands::add_named_tunnel,
            ipc::commands::provision_named_tunnel,
            ipc::commands::start_named_tunnel,
            ipc::commands::stop_named_tunnel,
            ipc::commands::named_tunnel_running,
            ipc::commands::remove_named_tunnel,
            ipc::commands::toggle_cors,
            ipc::commands::export_domains,
            ipc::commands::import_domains,
            ipc::commands::get_app_settings,
            ipc::commands::set_autostart,
            ipc::commands::set_start_hidden,
            ipc::commands::scan_ports,
            ipc::commands::scan_projects,
            ipc::commands::list_templates,
            ipc::commands::get_home_dir,
            ipc::commands::open_terminal,
            ipc::commands::open_folder,
            ipc::commands::import_nginx_config,
            ipc::commands::import_nginx_config_text,
            ipc::commands::validate_nginx_config,
            ipc::commands::export_nginx_config_to_project,
            ipc::commands::docker_check,
            ipc::commands::compose_status,
            ipc::commands::compose_up,
            ipc::commands::compose_down,
            ipc::commands::compose_restart,
            ipc::commands::compose_logs,
            ipc::commands::compose_save_file,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                use tauri::Manager;
                let state = app_handle.state::<AppState>();
                let _ = state.nginx.stop();
                tracing::info!("App exiting, stopped Nginx.");
            }
        });
}

/// Build the platform-specific sidecar binary filename.
/// Tauri names external binaries as `{name}-{target_triple}{exe_suffix}`.
pub fn sidecar_name(base: &str) -> String {
    let triple = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        "aarch64-pc-windows-msvc"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "x86_64-unknown-linux-gnu"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "aarch64-unknown-linux-gnu"
    } else {
        ""
    };
    let ext = std::env::consts::EXE_SUFFIX;
    if triple.is_empty() {
        format!("{}{}", base, ext)
    } else {
        format!("{}-{}{}", base, triple, ext)
    }
}

fn exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default()
}

pub fn resolve_nginx_exe() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        // Windows: use bundled sidecar binary
        let dir = exe_dir();
        let sidecar = sidecar_name("nginx");
        let fallback = format!("nginx{}", std::env::consts::EXE_SUFFIX);

        let candidates = [
            dir.join(&sidecar),
            dir.join(&fallback),
            std::path::PathBuf::from("src-tauri/binaries").join(&sidecar),
            std::path::PathBuf::from("binaries").join(&sidecar),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                tracing::info!("Found nginx at: {}", candidate.display());
                return candidate.clone();
            }
        }

        tracing::warn!("nginx sidecar not found, falling back to PATH");
        std::path::PathBuf::from(fallback)
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: use Homebrew nginx (brew install nginx)
        let candidates = [
            "/opt/homebrew/bin/nginx",  // Apple Silicon (ARM64)
            "/usr/local/bin/nginx",     // Intel (x86_64)
            "/opt/local/bin/nginx",     // MacPorts
        ];

        for path in &candidates {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                tracing::info!("Found nginx at: {}", p.display());
                return p;
            }
        }

        tracing::warn!("nginx not found in Homebrew paths. Run: brew install nginx");
        std::path::PathBuf::from("nginx")
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: use system nginx (apt/yum install nginx)
        let candidates = [
            "/usr/sbin/nginx",       // Debian/Ubuntu
            "/usr/bin/nginx",        // Some distros
            "/usr/local/sbin/nginx", // Custom install
            "/usr/local/bin/nginx",  // Custom install
        ];

        for path in &candidates {
            let p = std::path::PathBuf::from(path);
            if p.exists() {
                tracing::info!("Found nginx at: {}", p.display());
                return p;
            }
        }

        tracing::warn!("nginx not found. Run: sudo apt install nginx (or yum/pacman)");
        std::path::PathBuf::from("nginx")
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        std::path::PathBuf::from("nginx")
    }
}

pub fn resolve_cloudflared_exe() -> std::path::PathBuf {
    let dir = exe_dir();
    let sidecar = sidecar_name("cloudflared");
    let fallback = format!("cloudflared{}", std::env::consts::EXE_SUFFIX);

    let dev_binaries = dir
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("binaries"));

    let mut candidates = vec![dir.join(&sidecar)];
    if let Some(dev_dir) = dev_binaries {
        candidates.push(dev_dir.join(&sidecar));
    }

    tracing::info!(
        "resolve_cloudflared_exe: candidates={:?}",
        candidates
            .iter()
            .map(|p| format!("{} (exists={})", p.display(), p.exists()))
            .collect::<Vec<_>>()
    );

    for candidate in &candidates {
        if candidate.exists() {
            tracing::info!("Found cloudflared at: {}", candidate.display());
            return candidate.clone();
        }
    }

    tracing::warn!("cloudflared not found, falling back to PATH");
    std::path::PathBuf::from(fallback)
}
