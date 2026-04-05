use serde_json::Value;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn start_tailing(app_handle: AppHandle, log_path: PathBuf) {
    if !log_path.exists() {
        // Nginx might not have created it yet. Let's create an empty file.
        let _ = std::fs::write(&log_path, "");
    }

    tauri::async_runtime::spawn(async move {
        // Start from the end of the file to only capture "live" traffic
        let mut last_pos = std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0);

        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let metadata = match std::fs::metadata(&log_path) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let current_len = metadata.len();

            // Handle file rotation or truncation
            if current_len < last_pos {
                last_pos = 0;
            }

            if current_len > last_pos {
                match std::fs::File::open(&log_path) {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        if reader.seek(SeekFrom::Start(last_pos)).is_ok() {
                            let mut line = String::new();
                            while let Ok(bytes_read) = reader.read_line(&mut line) {
                                if bytes_read == 0 {
                                    break;
                                }
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                                        let _ = app_handle.emit("nginx_access_log", parsed);
                                    }
                                }
                                line.clear();
                            }
                            // Update last_pos to the actual current length after reading
                            last_pos = current_len;
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to open log file for polling: {}", e);
                    }
                }
            }
        }
    });
}
