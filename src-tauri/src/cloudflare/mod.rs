pub mod named_tunnel;

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize)]
struct TunnelReadyPayload {
    domain: String,
    url: String,
}

#[derive(Clone, serde::Serialize)]
struct TunnelErrorPayload {
    domain: String,
    error: String,
}

struct TunnelEntry {
    child: Child,
    /// Public URL đã được assign bởi TryCloudflare (nếu đã nhận được)
    public_url: Option<String>,
}

pub struct CloudflaredManager {
    processes: Arc<Mutex<HashMap<String, TunnelEntry>>>,
}

impl CloudflaredManager {
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(
        &self,
        app_handle: AppHandle,
        domain: String,
        upstream: String,
    ) -> anyhow::Result<()> {
        // Kiểm tra nếu tunnel đang chạy — emit lại URL nếu đã biết
        {
            let procs = self.processes.lock().unwrap();
            if let Some(entry) = procs.get(&domain) {
                if let Some(url) = &entry.public_url {
                    let _ = app_handle.emit(
                        "tunnel_ready",
                        TunnelReadyPayload {
                            domain: domain.clone(),
                            url: url.clone(),
                        },
                    );
                }
                return Ok(());
            }
        } // <- lock drop trước khi spawn

        let exe_path = crate::resolve_cloudflared_exe();
        if !exe_path.exists() {
            anyhow::bail!("cloudflared binary not found at {}", exe_path.display());
        }

        tracing::info!(
            "Spawning cloudflared: exe={} upstream={} cwd={:?}",
            exe_path.display(),
            upstream,
            std::env::current_dir()
        );

        // Spawn bên ngoài lock để tránh block caller khác
        let mut child = Command::new(&exe_path)
            .args(["tunnel", "--url", &upstream])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()?;

        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture cloudflared stderr"))?;

        {
            let mut procs = self.processes.lock().unwrap();
            // Double-check: tránh race nếu hai call đến cùng lúc
            if procs.contains_key(&domain) {
                let _ = child.kill();
                let _ = child.wait();
                return Ok(());
            }
            procs.insert(
                domain.clone(),
                TunnelEntry {
                    child,
                    public_url: None,
                },
            );
        }

        // Spawn reader thread sau khi entry đã vào map
        let processes = Arc::clone(&self.processes);
        let domain_clone = domain.clone();

        std::thread::Builder::new()
            .name(format!("cloudflared-stderr-{}", domain))
            .spawn(move || {
                read_stderr_for_url(stderr, &domain_clone, &app_handle, &processes);
            })
            .map_err(|e| anyhow::anyhow!("Failed to spawn stderr reader thread: {}", e))?;

        Ok(())
    }

    pub fn stop(&self, domain: &str) {
        let mut procs = self.processes.lock().unwrap();
        if let Some(mut entry) = procs.remove(domain) {
            let _ = entry.child.kill();
            // wait() để OS reclaim resources, tránh zombie process
            let _ = entry.child.wait();
            tracing::info!("Stopped cloudflared tunnel for {}", domain);
        }
    }

    pub fn stop_all(&self) {
        let mut procs = self.processes.lock().unwrap();
        for (domain, mut entry) in procs.drain() {
            let _ = entry.child.kill();
            let _ = entry.child.wait();
            tracing::info!("Stopped cloudflared tunnel for {}", domain);
        }
    }
}

impl Drop for CloudflaredManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

/// Đọc stderr của cloudflared để bắt TryCloudflare URL.
/// Khi tìm thấy URL, lưu vào map và emit event đến frontend.
/// Thread tự kết thúc khi pipe đóng (process bị kill hoặc crash).
fn read_stderr_for_url(
    stderr: impl std::io::Read,
    domain: &str,
    app_handle: &AppHandle,
    processes: &Mutex<HashMap<String, TunnelEntry>>,
) {
    let reader = BufReader::new(stderr);
    let mut pending_url: Option<String> = None;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("cloudflared [{}] stderr read error: {}", domain, e);
                break;
            }
        };

        tracing::debug!("cloudflared [{}]: {}", domain, line);

        // Bước 1: capture URL khi thấy
        if pending_url.is_none() {
            if let Some(url) = extract_trycloudflare_url(&line) {
                tracing::info!("cloudflared [{}] got URL: {}", domain, url);
                pending_url = Some(url);
                // Chưa emit — đợi confirmation
                continue;
            }
        }

        // Bước 2: đợi dòng confirm tunnel đã register xong
        if pending_url.is_some() {
            let lower = line.to_lowercase();
            let is_ready = lower.contains("registered")
                || lower.contains("connected")
                || lower.contains("your quick tunnel has been created");

            if is_ready {
                let url = pending_url.take().unwrap();
                tracing::info!("cloudflared [{}] tunnel confirmed ready: {}", domain, url);

                {
                    let mut procs = processes.lock().unwrap();
                    if let Some(entry) = procs.get_mut(domain) {
                        entry.public_url = Some(url.clone());
                    }
                }

                let _ = app_handle.emit(
                    "tunnel_ready",
                    TunnelReadyPayload {
                        domain: domain.to_string(),
                        url,
                    },
                );
            }
        }
    }

    tracing::debug!("cloudflared [{}] stderr pipe closed", domain);
}

/// Parse TryCloudflare URL từ một dòng log.
/// Cloudflare log format: `... | https://some-words.trycloudflare.com |...`
fn extract_trycloudflare_url(line: &str) -> Option<String> {
    let start = line.find("https://")?;
    let rest = &line[start..];

    // URL kết thúc tại whitespace, pipe, hoặc hết chuỗi
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '|')
        .unwrap_or(rest.len());

    let url = &rest[..end];

    // Chỉ chấp nhận trycloudflare.com domain
    if url.contains("trycloudflare.com") && url.starts_with("https://") {
        Some(url.to_string())
    } else {
        None
    }
}
