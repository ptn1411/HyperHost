use linemux::MuxedLines;
use serde_json::Value;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

pub fn start_tailing(app_handle: AppHandle, log_path: PathBuf) {
    if !log_path.exists() {
        // Nginx might not have created it yet. Let's create an empty file.
        let _ = std::fs::write(&log_path, "");
    }

    tauri::async_runtime::spawn(async move {
        let mut lines = MuxedLines::new().expect("Failed to create linemux");
        if let Err(e) = lines.add_file(&log_path).await {
            tracing::error!("linemux error adding file: {}", e);
            return;
        }

        while let Ok(Some(line)) = lines.next_line().await {
            let log_line = line.line();
            if let Ok(parsed) = serde_json::from_str::<Value>(log_line) {
                let _ = app_handle.emit("nginx_access_log", parsed);
            }
        }
    });
}
