//! Minimal Model Context Protocol (MCP) server exposing HyperHost tools over
//! stdio. Compatible with Claude Code, Cursor, and any MCP-capable client.
//!
//! The stdio transport is newline-delimited JSON-RPC 2.0: one JSON object per
//! line on stdin/stdout. `tracing` logs must go to stderr.

use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::state::AppState;

const PROTOCOL_VERSION: &str = "2024-11-05";
const SERVER_NAME: &str = "hyperhost";

pub fn serve(state: AppState) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("mcp: stdin read error: {}", e);
                break;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let req: Value = match serde_json::from_str(trimmed) {
            Ok(v) => v,
            Err(e) => {
                let err = error_response(Value::Null, -32700, &format!("parse error: {}", e));
                write_message(&mut out, &err)?;
                continue;
            }
        };

        // Notifications have no `id` and expect no response.
        let is_notification = req.get("id").is_none();
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let method = req.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let params = req.get("params").cloned().unwrap_or(Value::Null);

        let response = dispatch(&state, method, params);

        match response {
            DispatchResult::Response(r) => {
                if !is_notification {
                    let env = json!({ "jsonrpc": "2.0", "id": id, "result": r });
                    write_message(&mut out, &env)?;
                }
            }
            DispatchResult::Error { code, message } => {
                if !is_notification {
                    let env = error_response(id, code, &message);
                    write_message(&mut out, &env)?;
                }
            }
            DispatchResult::NoReply => {}
        }
    }
    Ok(())
}

enum DispatchResult {
    Response(Value),
    Error { code: i32, message: String },
    NoReply,
}

fn dispatch(state: &AppState, method: &str, params: Value) -> DispatchResult {
    match method {
        "initialize" => DispatchResult::Response(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "serverInfo": {
                "name": SERVER_NAME,
                "version": env!("CARGO_PKG_VERSION"),
            },
            "capabilities": {
                "tools": {},
            },
        })),
        "initialized" | "notifications/initialized" => DispatchResult::NoReply,
        "ping" => DispatchResult::Response(json!({})),
        "tools/list" => DispatchResult::Response(json!({ "tools": tool_definitions() })),
        "tools/call" => call_tool(state, params),
        _ => DispatchResult::Error {
            code: -32601,
            message: format!("method not found: {}", method),
        },
    }
}

fn call_tool(state: &AppState, params: Value) -> DispatchResult {
    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => {
            return DispatchResult::Error {
                code: -32602,
                message: "missing `name`".into(),
            }
        }
    };
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));

    match run_tool(state, &name, args) {
        Ok(value) => DispatchResult::Response(tool_result_ok(&value)),
        Err(e) => DispatchResult::Response(tool_result_err(&e.to_string())),
    }
}

fn tool_result_ok(value: &Value) -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string(value).unwrap_or_else(|_| "{}".into()),
        }],
        "isError": false,
    })
}

fn tool_result_err(msg: &str) -> Value {
    json!({
        "content": [{ "type": "text", "text": msg }],
        "isError": true,
    })
}

// ──────────────────────────── tool registry ──────────────────────────────

fn tool_definitions() -> Vec<Value> {
    vec![
        tool_def(
            "list_domains",
            "List all configured local HTTPS domains.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "add_domain",
            "Register a new local HTTPS domain. Domain must end with .test or .local.",
            json!({
                "type": "object",
                "properties": {
                    "domain": { "type": "string", "description": "e.g. myapp.test" },
                    "upstream": { "type": "string", "description": "e.g. http://127.0.0.1:3000" }
                },
                "required": ["domain", "upstream"]
            }),
        ),
        tool_def(
            "remove_domain",
            "Delete a local domain and its cert.",
            json!({
                "type": "object",
                "properties": { "domain": { "type": "string" } },
                "required": ["domain"]
            }),
        ),
        tool_def(
            "toggle_domain",
            "Enable or disable a domain.",
            json!({
                "type": "object",
                "properties": { "domain": { "type": "string" } },
                "required": ["domain"]
            }),
        ),
        tool_def(
            "toggle_cors",
            "Toggle permissive CORS headers for a domain.",
            json!({
                "type": "object",
                "properties": { "domain": { "type": "string" } },
                "required": ["domain"]
            }),
        ),
        tool_def(
            "nginx_status",
            "Check whether the local nginx proxy is running.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "nginx_start",
            "Start the nginx proxy.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "nginx_stop",
            "Stop the nginx proxy.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "nginx_reload",
            "Reload nginx config after changes.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "scan_ports",
            "List currently listening TCP ports with PID and process name.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "scan_projects",
            "Scan a directory tree for dev projects (Node/Rust/Python/Go/PHP).",
            json!({
                "type": "object",
                "properties": {
                    "root": { "type": "string", "description": "root folder (defaults to home)" },
                    "depth": { "type": "integer", "description": "recursion depth (default 3)" }
                }
            }),
        ),
        tool_def(
            "doctor",
            "Run health checks on a domain: cert validity, hosts entry, upstream reachability.",
            json!({
                "type": "object",
                "properties": { "domain": { "type": "string" } },
                "required": ["domain"]
            }),
        ),
        tool_def(
            "docker_check",
            "Check whether Docker CLI + daemon are available.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "compose_status",
            "List compose files + running services in a project directory.",
            json!({
                "type": "object",
                "properties": { "project_path": { "type": "string" } },
                "required": ["project_path"]
            }),
        ),
        tool_def(
            "compose_up",
            "Run `docker compose up -d` in a project.",
            json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string" },
                    "file": { "type": "string", "description": "specific compose file name" }
                },
                "required": ["project_path"]
            }),
        ),
        tool_def(
            "compose_down",
            "Run `docker compose down`.",
            json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string" },
                    "file": { "type": "string" }
                },
                "required": ["project_path"]
            }),
        ),
        tool_def(
            "compose_restart",
            "Run `docker compose restart`.",
            json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string" },
                    "file": { "type": "string" }
                },
                "required": ["project_path"]
            }),
        ),
        tool_def(
            "compose_logs",
            "Read docker compose logs.",
            json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string" },
                    "file": { "type": "string" },
                    "lines": { "type": "integer", "description": "default 200" }
                },
                "required": ["project_path"]
            }),
        ),
        tool_def(
            "compose_save_file",
            "Save a docker-compose YAML into a project directory.",
            json!({
                "type": "object",
                "properties": {
                    "project_path": { "type": "string" },
                    "file_name":    { "type": "string" },
                    "content":      { "type": "string" }
                },
                "required": ["project_path", "file_name", "content"]
            }),
        ),
        tool_def(
            "list_named_tunnels",
            "List registered Cloudflare named tunnels.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool_def(
            "add_named_tunnel",
            "Register a Cloudflare named tunnel (before provisioning).",
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "hostname": { "type": "string" },
                    "upstream": { "type": "string" }
                },
                "required": ["name", "hostname", "upstream"]
            }),
        ),
        tool_def(
            "open_domain",
            "Open https://<domain> in the user's default browser.",
            json!({
                "type": "object",
                "properties": { "domain": { "type": "string" } },
                "required": ["domain"]
            }),
        ),
    ]
}

fn tool_def(name: &str, description: &str, schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": schema,
    })
}

// ──────────────────────────── tool dispatcher ────────────────────────────

fn run_tool(state: &AppState, name: &str, args: Value) -> anyhow::Result<Value> {
    match name {
        "list_domains" => tool_list_domains(state),
        "add_domain" => tool_add_domain(state, &args),
        "remove_domain" => tool_remove_domain(state, &args),
        "toggle_domain" => tool_toggle_domain(state, &args),
        "toggle_cors" => tool_toggle_cors(state, &args),
        "nginx_status" => Ok(json!({ "running": state.nginx.is_running() })),
        "nginx_start" => {
            rebuild_nginx(state)?;
            state.nginx.start()?;
            Ok(json!({ "ok": true }))
        }
        "nginx_stop" => {
            state.nginx.stop()?;
            Ok(json!({ "ok": true }))
        }
        "nginx_reload" => {
            rebuild_nginx(state)?;
            state.nginx.reload()?;
            Ok(json!({ "ok": true }))
        }
        "scan_ports" => tool_scan_ports(),
        "scan_projects" => tool_scan_projects(&args),
        "doctor" => tool_doctor(state, &args),
        "docker_check" => {
            let s = crate::docker::check_docker();
            Ok(json!({
                "installed": s.installed,
                "version": s.version,
                "daemon_running": s.daemon_running,
            }))
        }
        "compose_status" => tool_compose_status(&args),
        "compose_up" => tool_compose_run(&args, "up"),
        "compose_down" => tool_compose_run(&args, "down"),
        "compose_restart" => tool_compose_run(&args, "restart"),
        "compose_logs" => tool_compose_logs(&args),
        "compose_save_file" => tool_compose_save_file(&args),
        "list_named_tunnels" => tool_list_named_tunnels(state),
        "add_named_tunnel" => tool_add_named_tunnel(state, &args),
        "open_domain" => tool_open_domain(&args),
        _ => anyhow::bail!("unknown tool: {}", name),
    }
}

fn str_arg(args: &Value, key: &str) -> anyhow::Result<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow::anyhow!("missing argument: {}", key))
}

fn opt_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

fn tool_list_domains(state: &AppState) -> anyhow::Result<Value> {
    let domains = state.db.list_domains()?;
    let rows: Vec<_> = domains
        .iter()
        .map(|d| {
            json!({
                "domain": d.domain,
                "upstream": d.upstream,
                "enabled": d.enabled,
                "cors_enabled": d.cors_enabled,
                "cert_expiry": d.cert_expiry,
                "project_path": d.project_path,
            })
        })
        .collect();
    Ok(json!({ "domains": rows }))
}

fn tool_add_domain(state: &AppState, args: &Value) -> anyhow::Result<Value> {
    let domain = str_arg(args, "domain")?;
    let upstream = str_arg(args, "upstream")?;
    if !domain.ends_with(".test") && !domain.ends_with(".local") {
        anyhow::bail!("Domain must end with .test or .local");
    }
    if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
        anyhow::bail!("Upstream must start with http:// or https://");
    }

    let (cert_pem, key_pem) = state
        .ca
        .issue_for_domain(&domain)
        .map_err(|e| anyhow::anyhow!("cert issue failed: {}", e))?;
    let cert_dir = state.paths.cert_dir();
    std::fs::create_dir_all(&cert_dir)?;
    std::fs::write(cert_dir.join(format!("{}.crt", domain)), &cert_pem)?;
    std::fs::write(cert_dir.join(format!("{}.key", domain)), &key_pem)?;

    let expiry = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(crate::cert::ca::CERT_VALIDITY_DAYS))
        .map(|d| d.to_rfc3339());
    let cfg = crate::db::DomainConfig {
        id: None,
        domain: domain.clone(),
        upstream: upstream.clone(),
        enabled: true,
        cors_enabled: false,
        cert_expiry: expiry,
        created_at: None,
        advanced_config: None,
        project_path: None,
        run_command: None,
    };
    state.db.upsert_domain(&cfg, &cert_pem, &key_pem)?;
    sync_and_reload(state)?;
    Ok(json!({ "ok": true, "url": format!("https://{}", domain) }))
}

fn tool_remove_domain(state: &AppState, args: &Value) -> anyhow::Result<Value> {
    let domain = str_arg(args, "domain")?;
    state.db.remove_domain(&domain)?;
    let cert_dir = state.paths.cert_dir();
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.crt", domain)));
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.key", domain)));
    sync_and_reload(state)?;
    Ok(json!({ "ok": true }))
}

fn tool_toggle_domain(state: &AppState, args: &Value) -> anyhow::Result<Value> {
    let domain = str_arg(args, "domain")?;
    let new_state = state.db.toggle_domain(&domain)?;
    sync_and_reload(state)?;
    Ok(json!({ "enabled": new_state }))
}

fn tool_toggle_cors(state: &AppState, args: &Value) -> anyhow::Result<Value> {
    let domain = str_arg(args, "domain")?;
    let enabled = state.db.toggle_cors(&domain)?;
    rebuild_nginx(state)?;
    Ok(json!({ "cors_enabled": enabled }))
}

fn tool_scan_ports() -> anyhow::Result<Value> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let ports = rt.block_on(crate::detect::ports::scan_listening_ports_detailed());
    let rows: Vec<_> = ports
        .iter()
        .map(|p| {
            json!({
                "port": p.port,
                "pid": p.pid,
                "process": p.process,
                "guess": crate::detect::ports::guess_framework(p.port),
            })
        })
        .collect();
    Ok(json!({ "ports": rows }))
}

fn tool_scan_projects(args: &Value) -> anyhow::Result<Value> {
    let root = match opt_str(args, "root") {
        Some(p) => std::path::PathBuf::from(p),
        None => dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?,
    };
    let depth = args
        .get("depth")
        .and_then(|v| v.as_u64())
        .unwrap_or(3) as usize;
    let projects = crate::detect::projects::scan_projects(&root, depth);
    Ok(json!({ "projects": projects }))
}

fn tool_doctor(state: &AppState, args: &Value) -> anyhow::Result<Value> {
    let domain = str_arg(args, "domain")?;
    let domains = state.db.list_domains()?;
    let cfg = domains
        .iter()
        .find(|d| d.domain == domain)
        .ok_or_else(|| anyhow::anyhow!("Domain not configured"))?;

    let cert_ok = cfg.cert_expiry.as_ref().map_or(false, |exp| {
        chrono::DateTime::parse_from_rfc3339(exp)
            .map(|dt| dt > chrono::Utc::now())
            .unwrap_or(false)
    });
    let hosts_ok = hosts_file_has(&domain);
    let upstream_ok = upstream_reachable(&cfg.upstream);
    let nginx_running = state.nginx.is_running();

    Ok(json!({
        "domain": domain,
        "enabled": cfg.enabled,
        "cert_valid": cert_ok,
        "cert_expiry": cfg.cert_expiry,
        "hosts_mapped": hosts_ok,
        "upstream_reachable": upstream_ok,
        "upstream": cfg.upstream,
        "nginx_running": nginx_running,
    }))
}

fn tool_compose_status(args: &Value) -> anyhow::Result<Value> {
    let p = str_arg(args, "project_path")?;
    let s = crate::docker::compose_status(std::path::Path::new(&p));
    let files: Vec<_> = s
        .files
        .iter()
        .map(|f| {
            let services: Vec<_> = f
                .services
                .iter()
                .map(|svc| {
                    json!({
                        "name": svc.name,
                        "image": svc.image,
                        "state": svc.state,
                        "status": svc.status,
                        "ports": svc.ports,
                    })
                })
                .collect();
            json!({ "path": f.path, "name": f.name, "services": services })
        })
        .collect();
    Ok(json!({ "files": files }))
}

fn tool_compose_run(args: &Value, action: &str) -> anyhow::Result<Value> {
    let p = str_arg(args, "project_path")?;
    let file = opt_str(args, "file");
    let path = std::path::Path::new(&p);
    let out = match action {
        "up" => crate::docker::compose_up(path, file.as_deref()),
        "down" => crate::docker::compose_down(path, file.as_deref()),
        "restart" => crate::docker::compose_restart(path, file.as_deref()),
        _ => return Err(anyhow::anyhow!("unknown action: {}", action)),
    }
    .map_err(anyhow::Error::msg)?;
    Ok(json!({ "ok": true, "output": out }))
}

fn tool_compose_logs(args: &Value) -> anyhow::Result<Value> {
    let p = str_arg(args, "project_path")?;
    let file = opt_str(args, "file");
    let lines = args
        .get("lines")
        .and_then(|v| v.as_u64())
        .unwrap_or(200) as usize;
    let out = crate::docker::compose_logs(std::path::Path::new(&p), file.as_deref(), lines)
        .map_err(anyhow::Error::msg)?;
    Ok(json!({ "output": out }))
}

fn tool_compose_save_file(args: &Value) -> anyhow::Result<Value> {
    let p = str_arg(args, "project_path")?;
    let name = str_arg(args, "file_name")?;
    let content = str_arg(args, "content")?;
    let saved = crate::docker::save_compose_file(std::path::Path::new(&p), &name, &content)
        .map_err(anyhow::Error::msg)?;
    Ok(json!({ "saved": saved.to_string_lossy() }))
}

fn tool_list_named_tunnels(state: &AppState) -> anyhow::Result<Value> {
    let tunnels = state.db.list_named_tunnels()?;
    let rows: Vec<_> = tunnels
        .iter()
        .map(|t| {
            json!({
                "tunnel_name": t.tunnel_name,
                "tunnel_id": t.tunnel_id,
                "hostname": t.hostname,
                "upstream": t.upstream,
                "enabled": t.enabled,
            })
        })
        .collect();
    Ok(json!({ "tunnels": rows }))
}

fn tool_add_named_tunnel(state: &AppState, args: &Value) -> anyhow::Result<Value> {
    let name = str_arg(args, "name")?;
    let hostname = str_arg(args, "hostname")?;
    let upstream = str_arg(args, "upstream")?;
    let cfg = crate::db::NamedTunnelConfig {
        id: None,
        tunnel_name: name.clone(),
        tunnel_id: None,
        credentials_path: None,
        hostname,
        upstream,
        enabled: true,
        created_at: None,
    };
    state.db.insert_named_tunnel(&cfg)?;
    Ok(json!({ "ok": true, "tunnel_name": name }))
}

fn tool_open_domain(args: &Value) -> anyhow::Result<Value> {
    let domain = str_arg(args, "domain")?;
    let url = format!("https://{}", domain);
    open_url(&url)?;
    Ok(json!({ "opened": url }))
}

// ──────────────────────────── helpers ────────────────────────────────────

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

fn sync_and_reload(state: &AppState) -> anyhow::Result<()> {
    let active = state.db.list_enabled_domains()?;
    crate::dns::hosts::sync_hosts(&active)?;
    rebuild_nginx(state)?;
    Ok(())
}

fn hosts_file_has(domain: &str) -> bool {
    let path = if cfg!(windows) {
        std::path::PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    } else {
        std::path::PathBuf::from("/etc/hosts")
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    content
        .lines()
        .any(|l| !l.trim_start().starts_with('#') && l.split_whitespace().any(|t| t == domain))
}

fn upstream_reachable(upstream: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    let rest = upstream.trim_start_matches("http://").trim_start_matches("https://");
    let hostport = rest.split('/').next().unwrap_or(rest);
    let host = hostport.split(':').next().unwrap_or("127.0.0.1");
    let port = if let Some((_, p)) = hostport.split_once(':') {
        p.parse::<u16>().ok()
    } else if upstream.starts_with("https://") {
        Some(443)
    } else {
        Some(80)
    };
    let Some(port) = port else { return false };
    let Ok(addrs) = (host, port).to_socket_addrs() else {
        return false;
    };
    for a in addrs {
        if TcpStream::connect_timeout(&a, Duration::from_millis(600)).is_ok() {
            return true;
        }
    }
    false
}

fn open_url(url: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}

// ──────────────────────────── wire format ────────────────────────────────

fn write_message(out: &mut impl Write, msg: &Value) -> io::Result<()> {
    let s = serde_json::to_string(msg).unwrap();
    out.write_all(s.as_bytes())?;
    out.write_all(b"\n")?;
    out.flush()
}

fn error_response(id: Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}
