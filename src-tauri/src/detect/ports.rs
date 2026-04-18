use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

const COMMON_PORTS: &[u16] = &[
    1337, 2368, 3000, 3001, 3002, 3003, 3030, 3333, 4000, 4200, 4321,
    5000, 5001, 5173, 5174, 5175, 5555, 6006, 7000, 7001, 7777,
    8000, 8001, 8080, 8081, 8082, 8088, 8090, 8443, 8787, 8888,
    9000, 9001, 9090, 9229, 10000, 11434, 19006,
];

#[derive(Debug, Clone)]
pub struct ListeningPort {
    pub port: u16,
    pub pid: Option<u32>,
    pub process: Option<String>,
}

/// Enumerate listening TCP ports on the loopback/any-address. Tries the OS
/// native tool first (`netstat -ano` on Windows, `lsof` on Unix) to pick up
/// ALL ports with PID + process name; falls back to probing a known list
/// if the native tool is unavailable or returns nothing useful.
pub async fn scan_listening_ports_detailed() -> Vec<ListeningPort> {
    let native = tokio::task::spawn_blocking(native_enumerate)
        .await
        .unwrap_or_default();
    if !native.is_empty() {
        return native;
    }
    let ports = probe_common_ports().await;
    ports
        .into_iter()
        .map(|port| ListeningPort { port, pid: None, process: None })
        .collect()
}

async fn probe_common_ports() -> Vec<u16> {
    let mut handles = Vec::with_capacity(COMMON_PORTS.len());
    for &port in COMMON_PORTS {
        handles.push(tokio::spawn(async move {
            match timeout(
                Duration::from_millis(250),
                TcpStream::connect(("127.0.0.1", port)),
            )
            .await
            {
                Ok(Ok(_)) => Some(port),
                _ => None,
            }
        }));
    }

    let mut open = Vec::new();
    for h in handles {
        if let Ok(Some(p)) = h.await {
            open.push(p);
        }
    }
    open.sort();
    open
}

#[cfg(target_os = "windows")]
fn native_enumerate() -> Vec<ListeningPort> {
    let output = match Command::new("netstat").args(["-ano", "-p", "TCP"]).output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut results: Vec<ListeningPort> = Vec::new();
    let mut seen: HashSet<u16> = HashSet::new();

    for line in text.lines() {
        let t = line.trim();
        if !t.contains("LISTENING") {
            continue;
        }
        let parts: Vec<&str> = t.split_whitespace().collect();
        // Windows netstat: Proto  LocalAddress  ForeignAddress  State  PID
        if parts.len() < 5 {
            continue;
        }
        let local = parts[1];
        if !is_loopback_or_any(local) {
            continue;
        }
        let port = match parse_trailing_port(local) {
            Some(p) => p,
            None => continue,
        };
        let pid: u32 = match parts[4].parse() {
            Ok(n) => n,
            Err(_) => continue,
        };
        if seen.insert(port) {
            results.push(ListeningPort { port, pid: Some(pid), process: None });
        }
    }

    if !results.is_empty() {
        let pid_to_name = windows_pid_to_name();
        for r in &mut results {
            if let Some(pid) = r.pid {
                if let Some(name) = pid_to_name.get(&pid) {
                    r.process = Some(name.clone());
                }
            }
        }
    }

    results.sort_by_key(|p| p.port);
    results
}

#[cfg(target_os = "windows")]
fn windows_pid_to_name() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    let Ok(out) = Command::new("tasklist").args(["/FO", "CSV", "/NH"]).output() else {
        return map;
    };
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        // "Image Name","PID","Session Name","Session#","Mem Usage"
        let fields: Vec<&str> = line.split("\",\"").collect();
        if fields.len() < 2 {
            continue;
        }
        let name_raw = fields[0].trim_start_matches('"');
        let name = name_raw.strip_suffix(".exe").unwrap_or(name_raw).to_string();
        let pid_raw = fields[1].trim_matches('"');
        if let Ok(pid) = pid_raw.parse::<u32>() {
            map.insert(pid, name);
        }
    }
    map
}

#[cfg(not(target_os = "windows"))]
fn native_enumerate() -> Vec<ListeningPort> {
    let output = match Command::new("lsof")
        .args(["-iTCP", "-sTCP:LISTEN", "-nP", "-F", "pcn"])
        .output()
    {
        Ok(o) if o.status.success() || !o.stdout.is_empty() => o,
        _ => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut results: Vec<ListeningPort> = Vec::new();
    let mut seen: HashSet<u16> = HashSet::new();
    let mut cur_pid: Option<u32> = None;
    let mut cur_cmd: Option<String> = None;

    for line in text.lines() {
        if line.is_empty() {
            continue;
        }
        let tag = &line[..1];
        let rest = &line[1..];
        match tag {
            "p" => {
                cur_pid = rest.parse().ok();
                cur_cmd = None;
            }
            "c" => cur_cmd = Some(rest.to_string()),
            "n" => {
                if let Some(port) = parse_lsof_name(rest) {
                    if seen.insert(port) {
                        results.push(ListeningPort {
                            port,
                            pid: cur_pid,
                            process: cur_cmd.clone(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    results.sort_by_key(|p| p.port);
    results
}

fn parse_trailing_port(s: &str) -> Option<u16> {
    s.rsplit(':').next()?.parse().ok()
}

fn is_loopback_or_any(local: &str) -> bool {
    local.starts_with("127.")
        || local.starts_with("0.0.0.0")
        || local.starts_with("[::1]")
        || local.starts_with("[::]")
        || local.starts_with("::1")
        || local == "*"
}

#[cfg(not(target_os = "windows"))]
fn parse_lsof_name(name: &str) -> Option<u16> {
    let core = name.split("->").next().unwrap_or(name);
    let port_str = core.rsplit(':').next()?;
    let addr_str = core.strip_suffix(&format!(":{}", port_str))?;
    let ok = addr_str == "*"
        || addr_str == "127.0.0.1"
        || addr_str.starts_with("127.")
        || addr_str == "[::1]"
        || addr_str == "[::]"
        || addr_str == "localhost";
    if !ok {
        return None;
    }
    port_str.parse().ok()
}

pub fn guess_framework(port: u16) -> Option<&'static str> {
    match port {
        1337 => Some("Strapi"),
        2368 => Some("Ghost"),
        3000 => Some("Next.js / Node / Rails"),
        3001 | 3002 | 3003 => Some("Node (alt)"),
        3030 => Some("Feathers / Meteor"),
        3333 => Some("NestJS (default)"),
        4000 => Some("Phoenix / Apollo"),
        4200 => Some("Angular CLI"),
        4321 => Some("Astro"),
        5000 => Some("Flask / .NET"),
        5173 | 5174 | 5175 => Some("Vite / SvelteKit"),
        5555 => Some("Prisma Studio"),
        6006 => Some("Storybook"),
        7000 | 7001 => Some("Cassandra / dev"),
        7777 => Some("dev server"),
        8000 => Some("Django / Laravel / FastAPI"),
        8001 => Some("Django (alt)"),
        8080 => Some("Go / Spring Boot / generic"),
        8081 | 8082 => Some("Go / generic (alt)"),
        8088 | 8090 => Some("dev server"),
        8443 => Some("HTTPS dev"),
        8787 => Some("R Shiny"),
        8888 => Some("Jupyter / dev"),
        9000 => Some("PHP-FPM / SonarQube"),
        9001 => Some("Supervisor / dev"),
        9090 => Some("Prometheus / dev"),
        9229 => Some("Node Inspector"),
        10000 => Some("Webmin / dev"),
        11434 => Some("Ollama"),
        19006 => Some("Expo web"),
        _ => None,
    }
}
