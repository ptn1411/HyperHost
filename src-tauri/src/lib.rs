pub mod cert;
pub mod db;
pub mod dns;
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

    let nginx_exe = resolve_nginx_exe();
    let nginx = nginx::NginxManager::new(nginx_exe, paths.nginx_conf(), paths.nginx_dir());

    Ok(AppState {
        paths,
        db,
        ca,
        nginx,
    })
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
                // Prevent app from closing and hide the window instead
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            ipc::commands::add_domain,
            ipc::commands::list_domains,
            ipc::commands::remove_domain,
            ipc::commands::toggle_domain,
            ipc::commands::install_ca,
            ipc::commands::ca_status,
            ipc::commands::nginx_status,
            ipc::commands::nginx_start,
            ipc::commands::nginx_stop,
            ipc::commands::get_nginx_log,
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

pub fn resolve_nginx_exe() -> std::path::PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();

    let candidates = [
        exe_dir.join("nginx-x86_64-pc-windows-msvc.exe"),
        exe_dir.join("nginx.exe"),
        std::path::PathBuf::from("src-tauri/binaries/nginx-x86_64-pc-windows-msvc.exe"),
        std::path::PathBuf::from("binaries/nginx-x86_64-pc-windows-msvc.exe"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            tracing::info!("Found nginx at: {}", candidate.display());
            return candidate.clone();
        }
    }

    tracing::warn!("nginx not found in expected locations, falling back to 'nginx' in PATH");
    std::path::PathBuf::from("nginx.exe")
}
