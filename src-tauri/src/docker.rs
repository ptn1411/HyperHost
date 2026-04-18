use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, serde::Serialize)]
pub struct DockerStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub daemon_running: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct ComposeStatus {
    pub files: Vec<ComposeFile>,
}

#[derive(Debug, serde::Serialize)]
pub struct ComposeFile {
    pub path: String,
    pub name: String,
    pub services: Vec<ComposeService>,
}

#[derive(Debug, serde::Serialize)]
pub struct ComposeService {
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
}

pub fn check_docker() -> DockerStatus {
    let installed = which("docker").is_some();
    if !installed {
        return DockerStatus { installed: false, version: None, daemon_running: false };
    }

    let version = Command::new("docker")
        .args(["version", "--format", "{{.Client.Version}}"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        });

    let daemon_running = Command::new("docker")
        .args(["info", "--format", "{{.ServerVersion}}"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    DockerStatus { installed: true, version, daemon_running }
}

pub fn find_compose_files(project_path: &Path) -> Vec<PathBuf> {
    let entries = match std::fs::read_dir(project_path) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut out: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            if !path.is_file() {
                return None;
            }
            let name = path.file_name()?.to_str()?.to_string();
            if matches_compose_pattern(&name) {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    out.sort();
    out
}

fn matches_compose_pattern(name: &str) -> bool {
    let lower = name.to_lowercase();
    let stem = lower
        .strip_suffix(".yml")
        .or_else(|| lower.strip_suffix(".yaml"));
    let stem = match stem {
        Some(s) => s,
        None => return false,
    };
    stem == "docker-compose"
        || stem == "compose"
        || stem.starts_with("docker-compose.")
        || stem.starts_with("compose.")
}

pub fn compose_status(project_path: &Path) -> ComposeStatus {
    let files = find_compose_files(project_path);
    let mut out = Vec::with_capacity(files.len());
    for path in files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let services = match Command::new("docker")
            .current_dir(project_path)
            .args(["compose", "-f", &name, "ps", "--all", "--format", "json"])
            .output()
        {
            Ok(o) if o.status.success() => parse_compose_ps(&String::from_utf8_lossy(&o.stdout)),
            _ => Vec::new(),
        };

        out.push(ComposeFile {
            path: path.to_string_lossy().to_string(),
            name,
            services,
        });
    }
    ComposeStatus { files: out }
}

pub fn compose_up(project_path: &Path, file: Option<&str>) -> Result<String, String> {
    run_compose(project_path, file, &["up", "-d"])
}

pub fn compose_down(project_path: &Path, file: Option<&str>) -> Result<String, String> {
    run_compose(project_path, file, &["down"])
}

pub fn compose_restart(project_path: &Path, file: Option<&str>) -> Result<String, String> {
    run_compose(project_path, file, &["restart"])
}

pub fn compose_logs(project_path: &Path, file: Option<&str>, lines: usize) -> Result<String, String> {
    run_compose(
        project_path,
        file,
        &["logs", "--tail", &lines.to_string(), "--no-color"],
    )
}

pub fn save_compose_file(
    project_path: &Path,
    file_name: &str,
    content: &str,
) -> Result<PathBuf, String> {
    if !project_path.is_dir() {
        return Err(format!(
            "Thư mục dự án không tồn tại: {}",
            project_path.display()
        ));
    }
    if !is_safe_compose_filename(file_name) {
        return Err(
            "Tên file không hợp lệ. Phải kết thúc bằng .yml/.yaml và chỉ dùng chữ/số/dấu chấm/gạch."
                .into(),
        );
    }
    let target = project_path.join(file_name);
    std::fs::write(&target, content).map_err(|e| format!("Không ghi được file: {}", e))?;
    Ok(target)
}

fn is_safe_compose_filename(name: &str) -> bool {
    if name.is_empty() || name.len() > 80 {
        return false;
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return false;
    }
    let lower = name.to_lowercase();
    if !(lower.ends_with(".yml") || lower.ends_with(".yaml")) {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
}

fn run_compose(project_path: &Path, file: Option<&str>, args: &[&str]) -> Result<String, String> {
    let files = find_compose_files(project_path);
    if files.is_empty() {
        return Err(format!(
            "Không tìm thấy file compose trong {}",
            project_path.display()
        ));
    }
    let resolved = match file {
        Some(f) => {
            if !is_safe_compose_filename(f) {
                return Err("Tên file compose không hợp lệ".into());
            }
            let target = project_path.join(f);
            if !target.is_file() {
                return Err(format!("File compose không tồn tại: {}", f));
            }
            Some(f.to_string())
        }
        None => None,
    };

    let mut cmd = Command::new("docker");
    cmd.current_dir(project_path).arg("compose");
    if let Some(f) = &resolved {
        cmd.arg("-f").arg(f);
    }
    for a in args {
        cmd.arg(a);
    }
    let output = cmd.output().map_err(|e| format!("Lỗi chạy docker: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stderr.is_empty() {
        stdout.clone()
    } else if stdout.is_empty() {
        stderr.clone()
    } else {
        format!("{}\n{}", stdout, stderr)
    };
    if output.status.success() {
        Ok(combined)
    } else {
        Err(combined.trim().to_string())
    }
}

fn parse_compose_ps(text: &str) -> Vec<ComposeService> {
    // `docker compose ps --format json` outputs either a single JSON array or
    // newline-delimited JSON objects depending on the docker version.
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    if trimmed.starts_with('[') {
        if let Ok(arr) = serde_json::from_str::<Vec<serde_json::Value>>(trimmed) {
            for v in arr {
                if let Some(s) = parse_one_service(&v) {
                    out.push(s);
                }
            }
        }
    } else {
        for line in trimmed.lines() {
            let l = line.trim();
            if l.is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(l) {
                if let Some(s) = parse_one_service(&v) {
                    out.push(s);
                }
            }
        }
    }
    out
}

fn parse_one_service(v: &serde_json::Value) -> Option<ComposeService> {
    let s = |k: &str| {
        v.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string()
    };
    let name = if !s("Service").is_empty() { s("Service") } else { s("Name") };
    if name.is_empty() {
        return None;
    }
    let publishers = format_publishers(v.get("Publishers"));
    let ports = if publishers.is_empty() { s("Ports") } else { publishers };
    Some(ComposeService {
        name,
        image: s("Image"),
        state: s("State"),
        status: s("Status"),
        ports,
    })
}

fn format_publishers(v: Option<&serde_json::Value>) -> String {
    let arr = match v.and_then(|x| x.as_array()) {
        Some(a) => a,
        None => return String::new(),
    };
    let mut parts = Vec::new();
    for entry in arr {
        let pub_port = entry.get("PublishedPort").and_then(|x| x.as_u64()).unwrap_or(0);
        let tgt_port = entry.get("TargetPort").and_then(|x| x.as_u64()).unwrap_or(0);
        if pub_port == 0 && tgt_port == 0 {
            continue;
        }
        if pub_port == 0 {
            parts.push(format!("{}", tgt_port));
        } else {
            parts.push(format!("{}->{}", pub_port, tgt_port));
        }
    }
    parts.join(", ")
}

fn which(cmd: &str) -> Option<PathBuf> {
    let exe = if cfg!(windows) { format!("{}.exe", cmd) } else { cmd.to_string() };
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let p = dir.join(&exe);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}
