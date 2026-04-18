use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, Color, Table,
};
use hyperhost_lib::state::AppState;
use serde_json::json;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "hyh",
    version = env!("CARGO_PKG_VERSION"),
    about = "⚡ HyperHost CLI — Local HTTPS domain manager",
    long_about = "Manage local virtual domains with HTTPS certificates.\nAdd domains like myapp.test that proxy to your dev server with trusted SSL.\n\nUsage: hyh add myapp.test http://127.0.0.1:3000"
)]
struct Cli {
    /// Emit JSON instead of human-formatted output (where supported)
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new local domain with HTTPS
    Add {
        /// Domain name (e.g. myapp.test)
        domain: String,
        /// Upstream URL (e.g. http://127.0.0.1:3000)
        upstream: String,
    },

    /// Remove a domain
    Remove {
        /// Domain name to remove
        domain: String,
    },

    /// List all configured domains
    List,

    /// Enable or disable a domain
    Toggle {
        /// Domain name to toggle
        domain: String,
    },

    /// Toggle CORS on/off for a domain
    Cors {
        /// Domain name
        domain: String,
    },

    /// Open https://<domain> in the default browser
    Open {
        /// Domain name
        domain: String,
    },

    /// Run health checks for a domain (cert / hosts / upstream)
    Doctor {
        /// Domain name
        domain: String,
    },

    /// Export all domain configs to JSON (stdout)
    Export,

    /// Import domain configs from a JSON file
    Import {
        /// Path to JSON file produced by `hyh export`
        file: PathBuf,
    },

    /// Manage nginx proxy
    Nginx {
        #[command(subcommand)]
        action: NginxAction,
    },

    /// Manage the local Certificate Authority
    Ca {
        #[command(subcommand)]
        action: CaAction,
    },

    /// Scan for common dev signals (listening ports / project folders)
    Scan {
        #[command(subcommand)]
        action: ScanAction,
    },

    /// Manage per-project Docker compose files
    Docker {
        #[command(subcommand)]
        action: DockerAction,
    },

    /// Manage Cloudflare named tunnels (fixed public domain)
    Tunnel {
        #[command(subcommand)]
        action: TunnelAction,
    },

    /// Run the MCP (Model Context Protocol) server over stdio
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },

    /// Print shell completion script
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(Subcommand)]
enum NginxAction {
    /// Start the nginx proxy
    Start,
    /// Stop the nginx proxy
    Stop,
    /// Reload nginx configuration
    Reload,
    /// Check nginx status
    Status,
    /// Show recent nginx error logs
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value = "20")]
        lines: usize,
    },
    /// Convert a production nginx config into a dev-friendly snippet
    Import {
        /// Path to production nginx config file
        file: PathBuf,
    },
    /// Validate a dev nginx snippet via `nginx -t`
    Validate {
        /// Path to dev-style nginx snippet
        file: PathBuf,
    },
    /// Generate a production-style nginx config from a registered dev domain
    Export {
        /// Dev domain already registered in HyperHost
        domain: String,
        /// Production domain to emit (e.g. example.com)
        prod_domain: String,
        /// Production upstream URL
        prod_upstream: String,
    },
}

#[derive(Subcommand)]
enum CaAction {
    /// Install CA into Windows trust store
    Install,
    /// Check CA installation status
    Status,
}

#[derive(Subcommand)]
enum ScanAction {
    /// List listening TCP ports with PID + process name
    Ports,
    /// Scan a folder tree for dev projects
    Projects {
        /// Root folder (defaults to home dir)
        #[arg(long)]
        root: Option<PathBuf>,
        /// Recursion depth (default 3)
        #[arg(long, default_value = "3")]
        depth: usize,
    },
}

#[derive(Subcommand)]
enum DockerAction {
    /// Check docker daemon + CLI availability
    Check,
    /// Show compose files + services in the project
    Status {
        /// Project directory (defaults to cwd)
        #[arg(long)]
        project: Option<PathBuf>,
    },
    /// `docker compose up -d` for a project
    Up {
        #[arg(long)]
        project: Option<PathBuf>,
        /// Specific compose file name (e.g. docker-compose.redis.yml)
        #[arg(long)]
        file: Option<String>,
    },
    /// `docker compose down`
    Down {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        file: Option<String>,
    },
    /// `docker compose restart`
    Restart {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        file: Option<String>,
    },
    /// `docker compose logs`
    Logs {
        #[arg(long)]
        project: Option<PathBuf>,
        #[arg(long)]
        file: Option<String>,
        #[arg(short, long, default_value = "200")]
        lines: usize,
    },
    /// Save a compose file into the project directory (reads stdin if no --content)
    Save {
        /// Target file name (e.g. docker-compose.yml)
        name: String,
        #[arg(long)]
        project: Option<PathBuf>,
        /// Inline content (otherwise read stdin)
        #[arg(long)]
        content: Option<String>,
    },
}

#[derive(Subcommand)]
enum TunnelAction {
    /// Check login + list configured tunnels
    Status,
    /// Run `cloudflared tunnel login` (opens browser)
    Login,
    /// List configured named tunnels
    List,
    /// Register a new tunnel in the local DB
    Add {
        /// Tunnel name (unique)
        name: String,
        /// Public hostname (e.g. api.example.com)
        hostname: String,
        /// Upstream URL (e.g. http://127.0.0.1:3000)
        upstream: String,
    },
    /// Create the tunnel on Cloudflare and write its config.yml
    Provision {
        /// Tunnel name
        name: String,
    },
    /// Run the tunnel in the foreground (Ctrl-C to stop)
    Run {
        /// Tunnel name
        name: String,
    },
    /// Remove a tunnel from the local DB (does not delete on Cloudflare)
    Remove {
        /// Tunnel name
        name: String,
    },
}

#[derive(Subcommand)]
enum McpAction {
    /// Start the MCP server on stdin/stdout (for Claude Code, Cursor, etc.)
    Serve,
    /// Print a suggested .claude/settings.json snippet to register this binary
    Snippet,
}

#[derive(Copy, Clone, ValueEnum)]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

impl CompletionShell {
    fn to_shell(self) -> Shell {
        match self {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::Powershell => Shell::PowerShell,
            CompletionShell::Elvish => Shell::Elvish,
        }
    }
}

// ──────────────────────────── entry point ────────────────────────────────

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(false)
        .without_time()
        .init();

    let cli = Cli::parse();
    let json = cli.json;

    if let Err(e) = run(cli) {
        if json {
            let payload = json!({ "ok": false, "error": e.to_string() });
            println!("{}", payload);
        } else {
            eprintln!("❌ Error: {}", e);
        }
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let json = cli.json;

    // Completions don't need state
    if let Commands::Completions { shell } = cli.command {
        let mut cmd = Cli::command();
        generate(shell.to_shell(), &mut cmd, "hyh", &mut io::stdout());
        return Ok(());
    }

    let state = hyperhost_lib::init_state()?;

    match cli.command {
        Commands::Add { domain, upstream } => cmd_add(&state, &domain, &upstream, json)?,
        Commands::Remove { domain } => cmd_remove(&state, &domain, json)?,
        Commands::List => cmd_list(&state, json)?,
        Commands::Toggle { domain } => cmd_toggle(&state, &domain, json)?,
        Commands::Cors { domain } => cmd_cors(&state, &domain, json)?,
        Commands::Open { domain } => cmd_open(&state, &domain, json)?,
        Commands::Doctor { domain } => cmd_doctor(&state, &domain, json)?,
        Commands::Export => cmd_export(&state, json)?,
        Commands::Import { file } => cmd_import(&state, &file, json)?,
        Commands::Nginx { action } => cmd_nginx(&state, action, json)?,
        Commands::Ca { action } => cmd_ca(&state, action, json)?,
        Commands::Scan { action } => cmd_scan(action, json)?,
        Commands::Docker { action } => cmd_docker(action, json)?,
        Commands::Tunnel { action } => cmd_tunnel(&state, action, json)?,
        Commands::Mcp { action } => cmd_mcp(state, action, json)?,
        Commands::Completions { .. } => unreachable!(),
    }

    Ok(())
}

// ──────────────────────────── domain commands ────────────────────────────

fn cmd_add(
    state: &AppState,
    domain: &str,
    upstream: &str,
    json: bool,
) -> anyhow::Result<()> {
    if !domain.ends_with(".test") && !domain.ends_with(".local") {
        anyhow::bail!("Domain must end with .test or .local");
    }
    if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
        anyhow::bail!("Upstream must start with http:// or https://");
    }

    if !json {
        println!("🔐 Issuing certificate for {}...", domain);
    }

    let cert_dir = state.paths.cert_dir();
    std::fs::create_dir_all(&cert_dir)?;

    let (cert_pem, key_pem) = match state.ca.issue_for_domain(domain) {
        Ok((cert, key)) => {
            std::fs::write(cert_dir.join(format!("{}.crt", domain)), &cert)?;
            std::fs::write(cert_dir.join(format!("{}.key", domain)), &key)?;
            if !json {
                println!("  ✓ Certificate issued (rcgen)");
            }
            (cert, key)
        }
        Err(rcgen_err) => {
            if !json {
                println!("  ⚠ rcgen failed: {}, trying mkcert...", rcgen_err);
            }
            let mkcert = hyperhost_lib::cert::mkcert::MkcertRunner::find()
                .ok_or_else(|| anyhow::anyhow!("rcgen failed and mkcert not found"))?;
            mkcert.issue_for_domain(domain, &cert_dir)?;
            let cert = std::fs::read_to_string(cert_dir.join(format!("{}.crt", domain)))?;
            let key = std::fs::read_to_string(cert_dir.join(format!("{}.key", domain)))?;
            if !json {
                println!("  ✓ Certificate issued (mkcert fallback)");
            }
            (cert, key)
        }
    };

    let expiry = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(hyperhost_lib::cert::ca::CERT_VALIDITY_DAYS))
        .map(|d| d.to_rfc3339());

    let cfg = hyperhost_lib::db::DomainConfig {
        id: None,
        domain: domain.to_string(),
        upstream: upstream.to_string(),
        enabled: true,
        cors_enabled: false,
        cert_expiry: expiry.clone(),
        created_at: None,
        advanced_config: None,
        project_path: None,
        run_command: None,
    };
    state.db.upsert_domain(&cfg, &cert_pem, &key_pem)?;

    let active = state.db.list_enabled_domains()?;
    hyperhost_lib::dns::hosts::sync_hosts(&active)?;
    rebuild_nginx(state)?;

    if json {
        println!(
            "{}",
            json!({
                "ok": true,
                "domain": domain,
                "upstream": upstream,
                "url": format!("https://{}", domain),
                "cert_expiry": expiry,
            })
        );
    } else {
        println!("📦 Saved to database");
        println!("📝 Hosts file updated");
        println!("🔄 nginx config regenerated");
        println!("\n✅ https://{} → {}", domain, upstream);
    }
    Ok(())
}

fn cmd_remove(state: &AppState, domain: &str, json: bool) -> anyhow::Result<()> {
    state.db.remove_domain(domain)?;

    let cert_dir = state.paths.cert_dir();
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.crt", domain)));
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.key", domain)));

    let active = state.db.list_enabled_domains()?;
    hyperhost_lib::dns::hosts::sync_hosts(&active)?;
    rebuild_nginx(state)?;

    if json {
        println!("{}", json!({ "ok": true, "domain": domain, "removed": true }));
    } else {
        println!("✅ Removed {}", domain);
    }
    Ok(())
}

fn cmd_list(state: &AppState, json: bool) -> anyhow::Result<()> {
    let domains = state.db.list_domains()?;

    if json {
        let rows: Vec<_> = domains
            .iter()
            .map(|d| {
                let cert_ok = d.cert_expiry.as_ref().map_or(false, |exp| {
                    chrono::DateTime::parse_from_rfc3339(exp)
                        .map(|dt| dt > chrono::Utc::now())
                        .unwrap_or(false)
                });
                json!({
                    "domain": d.domain,
                    "upstream": d.upstream,
                    "enabled": d.enabled,
                    "cors_enabled": d.cors_enabled,
                    "cert_valid": cert_ok,
                    "cert_expiry": d.cert_expiry,
                    "project_path": d.project_path,
                    "run_command": d.run_command,
                })
            })
            .collect();
        println!("{}", json!({ "ok": true, "domains": rows }));
        return Ok(());
    }

    if domains.is_empty() {
        println!("No domains configured. Use 'hyh add <domain> <upstream>' to get started.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Domain").fg(Color::Cyan).add_attribute(Attribute::Bold),
            Cell::new("Upstream").fg(Color::Cyan),
            Cell::new("Cert").fg(Color::Cyan),
        ]);

    for d in &domains {
        let status = if d.enabled { "🟢" } else { "⚫" };
        let cert_ok = d.cert_expiry.as_ref().map_or(false, |exp| {
            chrono::DateTime::parse_from_rfc3339(exp)
                .map(|dt| dt > chrono::Utc::now())
                .unwrap_or(false)
        });
        let cert_label = if cert_ok { "✓ valid" } else { "✗ expired" };

        table.add_row(vec![
            Cell::new(status),
            Cell::new(&d.domain).fg(Color::White).add_attribute(Attribute::Bold),
            Cell::new(&d.upstream).fg(Color::DarkGrey),
            if cert_ok {
                Cell::new(cert_label).fg(Color::Green)
            } else {
                Cell::new(cert_label).fg(Color::Red)
            },
        ]);
    }

    println!("{table}");
    println!("\n  {} domain(s) total", domains.len());
    Ok(())
}

fn cmd_toggle(state: &AppState, domain: &str, json: bool) -> anyhow::Result<()> {
    let new_state = state.db.toggle_domain(domain)?;
    let active = state.db.list_enabled_domains()?;
    hyperhost_lib::dns::hosts::sync_hosts(&active)?;
    rebuild_nginx(state)?;

    if json {
        println!(
            "{}",
            json!({ "ok": true, "domain": domain, "enabled": new_state })
        );
    } else {
        let icon = if new_state { "🟢 enabled" } else { "⚫ disabled" };
        println!("✅ {} → {}", domain, icon);
    }
    Ok(())
}

fn cmd_cors(state: &AppState, domain: &str, json: bool) -> anyhow::Result<()> {
    let enabled = state.db.toggle_cors(domain)?;
    rebuild_nginx(state)?;
    if json {
        println!(
            "{}",
            json!({ "ok": true, "domain": domain, "cors_enabled": enabled })
        );
    } else {
        println!(
            "✅ CORS on {} → {}",
            domain,
            if enabled { "enabled" } else { "disabled" }
        );
    }
    Ok(())
}

fn cmd_open(_state: &AppState, domain: &str, json: bool) -> anyhow::Result<()> {
    let url = format!("https://{}", domain);
    open_url(&url)?;
    if json {
        println!("{}", json!({ "ok": true, "opened": url }));
    } else {
        println!("🌐 Opened {}", url);
    }
    Ok(())
}

fn cmd_doctor(state: &AppState, domain: &str, json: bool) -> anyhow::Result<()> {
    let domains = state.db.list_domains()?;
    let cfg = domains
        .iter()
        .find(|d| d.domain == domain)
        .ok_or_else(|| anyhow::anyhow!("Domain not configured: {}", domain))?;

    // Cert check
    let cert_ok = cfg.cert_expiry.as_ref().map_or(false, |exp| {
        chrono::DateTime::parse_from_rfc3339(exp)
            .map(|dt| dt > chrono::Utc::now())
            .unwrap_or(false)
    });

    // Hosts file check
    let hosts_ok = hosts_file_has(domain);

    // Upstream reachability (HTTP HEAD via TcpStream to parsed port)
    let upstream_ok = upstream_reachable(&cfg.upstream);

    // nginx running
    let nginx_running = state.nginx.is_running();

    if json {
        println!(
            "{}",
            json!({
                "ok": true,
                "domain": domain,
                "checks": {
                    "cert_valid": cert_ok,
                    "cert_expiry": cfg.cert_expiry,
                    "hosts_mapped": hosts_ok,
                    "upstream_reachable": upstream_ok,
                    "upstream": cfg.upstream,
                    "nginx_running": nginx_running,
                    "enabled": cfg.enabled,
                }
            })
        );
    } else {
        println!("🩺 Doctor report for {}", domain);
        println!("  {} Enabled:            {}", mark(cfg.enabled), cfg.enabled);
        println!("  {} Certificate valid:  {}", mark(cert_ok), cert_ok);
        if let Some(e) = &cfg.cert_expiry {
            println!("    expiry: {}", e);
        }
        println!("  {} Hosts file mapped:  {}", mark(hosts_ok), hosts_ok);
        println!(
            "  {} Upstream reachable: {}  ({})",
            mark(upstream_ok),
            upstream_ok,
            cfg.upstream
        );
        println!("  {} nginx running:      {}", mark(nginx_running), nginx_running);
    }
    Ok(())
}

fn cmd_export(state: &AppState, json: bool) -> anyhow::Result<()> {
    let domains = state.db.list_domains()?;
    let out = serde_json::to_string_pretty(&json!({
        "version": 1,
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "domains": domains,
    }))?;
    if json {
        // emit as a single line JSON wrapper
        println!("{}", out);
    } else {
        println!("{}", out);
    }
    Ok(())
}

fn cmd_import(state: &AppState, file: &Path, json: bool) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(file)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    let list = parsed
        .get("domains")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Invalid export: missing `domains` array"))?;

    let mut imported = 0usize;
    for item in list {
        let cfg: hyperhost_lib::db::DomainConfig = serde_json::from_value(item.clone())?;
        // Re-issue cert locally
        let (cert_pem, key_pem) = state
            .ca
            .issue_for_domain(&cfg.domain)
            .map_err(|e| anyhow::anyhow!("Issue cert for {}: {}", cfg.domain, e))?;
        let cert_dir = state.paths.cert_dir();
        std::fs::create_dir_all(&cert_dir)?;
        std::fs::write(cert_dir.join(format!("{}.crt", cfg.domain)), &cert_pem)?;
        std::fs::write(cert_dir.join(format!("{}.key", cfg.domain)), &key_pem)?;
        state.db.upsert_domain(&cfg, &cert_pem, &key_pem)?;
        imported += 1;
    }

    let active = state.db.list_enabled_domains()?;
    hyperhost_lib::dns::hosts::sync_hosts(&active)?;
    rebuild_nginx(state)?;

    if json {
        println!("{}", json!({ "ok": true, "imported": imported }));
    } else {
        println!("✅ Imported {} domain(s)", imported);
    }
    Ok(())
}

// ──────────────────────────── nginx ──────────────────────────────────────

fn cmd_nginx(state: &AppState, action: NginxAction, json: bool) -> anyhow::Result<()> {
    match action {
        NginxAction::Start => {
            rebuild_nginx(state)?;
            state.nginx.start()?;
            out_ok(json, "nginx", "started");
        }
        NginxAction::Stop => {
            state.nginx.stop()?;
            out_ok(json, "nginx", "stopped");
        }
        NginxAction::Reload => {
            rebuild_nginx(state)?;
            state.nginx.reload()?;
            out_ok(json, "nginx", "reloaded");
        }
        NginxAction::Status => {
            let running = state.nginx.is_running();
            if json {
                println!("{}", json!({ "ok": true, "nginx_running": running }));
            } else {
                println!("nginx: {}", if running { "🟢 running" } else { "⚫ stopped" });
            }
        }
        NginxAction::Logs { lines } => {
            let log_path = state.paths.nginx_logs().join("error.log");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let all: Vec<&str> = content.lines().collect();
            let start = all.len().saturating_sub(lines);
            if json {
                let out: Vec<&str> = all[start..].to_vec();
                println!("{}", json!({ "ok": true, "lines": out }));
            } else {
                for line in &all[start..] {
                    println!("{}", line);
                }
            }
        }
        NginxAction::Import { file } => {
            let content = std::fs::read_to_string(&file)?;
            match hyperhost_lib::nginx::import::convert_prod_to_dev(&content) {
                Ok(imp) => {
                    if json {
                        println!(
                            "{}",
                            json!({
                                "ok": true,
                                "advanced_config": imp.advanced_config,
                                "suggested_upstream": imp.suggested_upstream,
                                "server_name": imp.server_name,
                            })
                        );
                    } else {
                        println!("✅ Imported (server_name: {:?})", imp.server_name);
                        if let Some(u) = &imp.suggested_upstream {
                            println!("   upstream suggestion: {}", u);
                        }
                        println!("\n── dev snippet ─────────────────");
                        println!("{}", imp.advanced_config);
                    }
                }
                Err(e) => anyhow::bail!("Import failed: {}", e),
            }
        }
        NginxAction::Validate { file } => {
            let content = std::fs::read_to_string(&file)?;
            let nginx_exe = hyperhost_lib::resolve_nginx_exe();
            match hyperhost_lib::nginx::import::validate_config(&content, &nginx_exe) {
                Ok(msg) => {
                    if json {
                        println!("{}", json!({ "ok": true, "message": msg }));
                    } else {
                        println!("✅ {}", msg);
                    }
                }
                Err(e) => anyhow::bail!("Invalid: {}", e),
            }
        }
        NginxAction::Export {
            domain,
            prod_domain,
            prod_upstream,
        } => {
            let domains = state.db.list_domains()?;
            let cfg = domains
                .iter()
                .find(|d| d.domain == domain)
                .ok_or_else(|| anyhow::anyhow!("Domain not configured: {}", domain))?;
            let dev_cfg = cfg.advanced_config.clone().unwrap_or_default();
            let out = hyperhost_lib::nginx::import::convert_dev_to_prod(
                &dev_cfg,
                &prod_domain,
                &prod_upstream,
            );
            if json {
                println!("{}", json!({ "ok": true, "nginx_config": out }));
            } else {
                println!("{}", out);
            }
        }
    }
    Ok(())
}

fn cmd_ca(state: &AppState, action: CaAction, json: bool) -> anyhow::Result<()> {
    let ca_cert = state.paths.ca_cert();
    match action {
        CaAction::Install => {
            let result = {
                #[cfg(target_os = "windows")]
                { hyperhost_lib::cert::windows_store::install_ca(&ca_cert) }
                #[cfg(target_os = "macos")]
                { hyperhost_lib::cert::macos_store::install_ca(&ca_cert) }
                #[cfg(target_os = "linux")]
                { hyperhost_lib::cert::linux_store::install_ca(&ca_cert) }
                #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
                { Err(anyhow::anyhow!("CA installation not supported on this platform")) }
            };
            result?;

            if let Some(mkcert) = hyperhost_lib::cert::mkcert::MkcertRunner::find() {
                match mkcert.install_ca() {
                    Ok(_) => {
                        if !json {
                            println!("  ✓ mkcert: Firefox NSS trusted");
                        }
                    }
                    Err(e) => {
                        if !json {
                            println!("  ⚠ mkcert -install failed: {}", e);
                        }
                    }
                }
            }

            if json {
                println!("{}", json!({ "ok": true, "installed": true }));
            } else {
                println!("\n✅ CA installed successfully");
            }
        }
        CaAction::Status => {
            let installed = {
                #[cfg(target_os = "windows")]
                { hyperhost_lib::cert::windows_store::is_ca_installed(&ca_cert) }
                #[cfg(target_os = "macos")]
                { hyperhost_lib::cert::macos_store::is_ca_installed(&ca_cert) }
                #[cfg(target_os = "linux")]
                { hyperhost_lib::cert::linux_store::is_ca_installed(&ca_cert) }
                #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
                { false }
            };
            let fp = state.ca.fingerprint();
            if json {
                println!(
                    "{}",
                    json!({ "ok": true, "installed": installed, "fingerprint": fp })
                );
            } else {
                println!(
                    "CA: {}",
                    if installed { "🟢 installed & trusted" } else { "⚫ not installed" }
                );
                if let Some(fp) = fp {
                    println!("SHA-256: {}", fp);
                }
            }
        }
    }
    Ok(())
}

// ──────────────────────────── scan ───────────────────────────────────────

fn cmd_scan(action: ScanAction, json: bool) -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    match action {
        ScanAction::Ports => {
            let ports = rt.block_on(hyperhost_lib::detect::ports::scan_listening_ports_detailed());
            if json {
                let rows: Vec<_> = ports
                    .iter()
                    .map(|p| {
                        json!({
                            "port": p.port,
                            "pid": p.pid,
                            "process": p.process,
                            "guess": hyperhost_lib::detect::ports::guess_framework(p.port),
                        })
                    })
                    .collect();
                println!("{}", json!({ "ok": true, "ports": rows }));
            } else {
                if ports.is_empty() {
                    println!("No listening ports detected.");
                    return Ok(());
                }
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_header(vec!["Port", "Process", "PID", "Guess"]);
                for p in &ports {
                    table.add_row(vec![
                        Cell::new(p.port).fg(Color::Cyan).add_attribute(Attribute::Bold),
                        Cell::new(p.process.clone().unwrap_or_else(|| "—".into())),
                        Cell::new(p.pid.map(|p| p.to_string()).unwrap_or_else(|| "—".into())),
                        Cell::new(
                            hyperhost_lib::detect::ports::guess_framework(p.port)
                                .unwrap_or("—")
                                .to_string(),
                        )
                        .fg(Color::DarkGrey),
                    ]);
                }
                println!("{table}");
            }
        }
        ScanAction::Projects { root, depth } => {
            let root = match root {
                Some(p) => p,
                None => dirs::home_dir().ok_or_else(|| anyhow::anyhow!("no home dir"))?,
            };
            let projects = hyperhost_lib::detect::projects::scan_projects(&root, depth);
            if json {
                println!("{}", json!({ "ok": true, "projects": projects }));
            } else {
                if projects.is_empty() {
                    println!("No projects found under {}", root.display());
                    return Ok(());
                }
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_header(vec!["Kind", "Name", "Domain", "Port", "Path"]);
                for p in &projects {
                    table.add_row(vec![
                        Cell::new(&p.kind).fg(Color::Cyan),
                        Cell::new(&p.name).add_attribute(Attribute::Bold),
                        Cell::new(&p.suggested_domain),
                        Cell::new(p.suggested_port),
                        Cell::new(&p.path).fg(Color::DarkGrey),
                    ]);
                }
                println!("{table}");
                println!("\n  {} project(s)", projects.len());
            }
        }
    }
    Ok(())
}

// ──────────────────────────── docker ─────────────────────────────────────

fn cmd_docker(action: DockerAction, json: bool) -> anyhow::Result<()> {
    use hyperhost_lib::docker;

    match action {
        DockerAction::Check => {
            let s = docker::check_docker();
            if json {
                println!(
                    "{}",
                    json!({
                        "ok": true,
                        "installed": s.installed,
                        "version": s.version,
                        "daemon_running": s.daemon_running,
                    })
                );
            } else {
                println!("Docker: installed={} version={:?} daemon_running={}",
                    s.installed, s.version, s.daemon_running);
            }
        }
        DockerAction::Status { project } => {
            let p = project_path(project)?;
            let s = docker::compose_status(&p);
            if json {
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
                        json!({
                            "path": f.path,
                            "name": f.name,
                            "services": services,
                        })
                    })
                    .collect();
                println!("{}", json!({ "ok": true, "files": files }));
            } else {
                if s.files.is_empty() {
                    println!("No compose files found in {}", p.display());
                    return Ok(());
                }
                for f in &s.files {
                    println!("📄 {}", f.name);
                    if f.services.is_empty() {
                        println!("   (no services running)");
                        continue;
                    }
                    for svc in &f.services {
                        println!("   • {}  {}  [{}]", svc.name, svc.image, svc.state);
                        if !svc.ports.is_empty() {
                            println!("       ports: {}", svc.ports);
                        }
                    }
                }
            }
        }
        DockerAction::Up { project, file } => {
            let p = project_path(project)?;
            let out = docker::compose_up(&p, file.as_deref()).map_err(anyhow::Error::msg)?;
            print_compose_output("up", &out, json);
        }
        DockerAction::Down { project, file } => {
            let p = project_path(project)?;
            let out = docker::compose_down(&p, file.as_deref()).map_err(anyhow::Error::msg)?;
            print_compose_output("down", &out, json);
        }
        DockerAction::Restart { project, file } => {
            let p = project_path(project)?;
            let out = docker::compose_restart(&p, file.as_deref()).map_err(anyhow::Error::msg)?;
            print_compose_output("restart", &out, json);
        }
        DockerAction::Logs { project, file, lines } => {
            let p = project_path(project)?;
            let out = docker::compose_logs(&p, file.as_deref(), lines)
                .map_err(anyhow::Error::msg)?;
            if json {
                println!("{}", json!({ "ok": true, "output": out }));
            } else {
                println!("{}", out);
            }
        }
        DockerAction::Save { name, project, content } => {
            let p = project_path(project)?;
            let body = match content {
                Some(c) => c,
                None => read_stdin()?,
            };
            let saved = docker::save_compose_file(&p, &name, &body)
                .map_err(anyhow::Error::msg)?;
            if json {
                println!(
                    "{}",
                    json!({ "ok": true, "saved": saved.to_string_lossy() })
                );
            } else {
                println!("✅ Saved {}", saved.display());
            }
        }
    }
    Ok(())
}

// ──────────────────────────── mcp ────────────────────────────────────────

fn cmd_mcp(state: AppState, action: McpAction, json: bool) -> anyhow::Result<()> {
    match action {
        McpAction::Serve => hyperhost_lib::mcp::serve(state),
        McpAction::Snippet => {
            let exe = std::env::current_exe()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "hyh".into());
            let snippet = json!({
                "mcpServers": {
                    "hyperhost": {
                        "command": exe,
                        "args": ["mcp", "serve"]
                    }
                }
            });
            if json {
                println!("{}", snippet);
            } else {
                println!("# Add to ~/.claude/settings.json (or your MCP client config):");
                println!("{}", serde_json::to_string_pretty(&snippet).unwrap());
            }
            Ok(())
        }
    }
}

// ──────────────────────────── tunnel ─────────────────────────────────────

fn cmd_tunnel(state: &AppState, action: TunnelAction, json: bool) -> anyhow::Result<()> {
    use hyperhost_lib::cloudflare::named_tunnel::NamedTunnelManager;

    let exe = hyperhost_lib::resolve_cloudflared_exe();

    match action {
        TunnelAction::Status => {
            let logged_in = NamedTunnelManager::is_logged_in();
            let tunnels = state.db.list_named_tunnels()?;
            if json {
                let rows: Vec<_> = tunnels
                    .iter()
                    .map(|t| {
                        json!({
                            "tunnel_name": t.tunnel_name,
                            "hostname": t.hostname,
                            "upstream": t.upstream,
                            "tunnel_id": t.tunnel_id,
                            "credentials_path": t.credentials_path,
                            "enabled": t.enabled,
                        })
                    })
                    .collect();
                println!(
                    "{}",
                    json!({ "ok": true, "logged_in": logged_in, "tunnels": rows })
                );
            } else {
                println!(
                    "Cloudflare login: {}",
                    if logged_in { "🟢 logged in" } else { "⚫ not logged in" }
                );
                if tunnels.is_empty() {
                    println!("\nNo named tunnels configured.");
                } else {
                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_header(vec!["Name", "Hostname", "Upstream", "Provisioned"]);
                    for t in &tunnels {
                        table.add_row(vec![
                            Cell::new(&t.tunnel_name).add_attribute(Attribute::Bold),
                            Cell::new(&t.hostname),
                            Cell::new(&t.upstream).fg(Color::DarkGrey),
                            Cell::new(if t.tunnel_id.is_some() { "✓" } else { "—" }),
                        ]);
                    }
                    println!("\n{table}");
                }
            }
        }
        TunnelAction::Login => {
            NamedTunnelManager::login(&exe)?;
            out_ok(json, "cloudflared", "logged in");
        }
        TunnelAction::List => {
            let tunnels = state.db.list_named_tunnels()?;
            if json {
                let rows: Vec<_> = tunnels
                    .iter()
                    .map(|t| {
                        json!({
                            "tunnel_name": t.tunnel_name,
                            "hostname": t.hostname,
                            "upstream": t.upstream,
                            "tunnel_id": t.tunnel_id,
                            "enabled": t.enabled,
                        })
                    })
                    .collect();
                println!("{}", json!({ "ok": true, "tunnels": rows }));
            } else if tunnels.is_empty() {
                println!("No named tunnels configured.");
            } else {
                for t in &tunnels {
                    let mark = if t.tunnel_id.is_some() { "✓" } else { "…" };
                    println!("{} {}  {} → {}", mark, t.tunnel_name, t.hostname, t.upstream);
                }
            }
        }
        TunnelAction::Add {
            name,
            hostname,
            upstream,
        } => {
            if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
                anyhow::bail!("Upstream must start with http:// or https://");
            }
            let cfg = hyperhost_lib::db::NamedTunnelConfig {
                id: None,
                tunnel_name: name.clone(),
                tunnel_id: None,
                credentials_path: None,
                hostname: hostname.clone(),
                upstream: upstream.clone(),
                enabled: true,
                created_at: None,
            };
            state.db.insert_named_tunnel(&cfg)?;
            if json {
                println!(
                    "{}",
                    json!({
                        "ok": true, "tunnel_name": name,
                        "hostname": hostname, "upstream": upstream,
                    })
                );
            } else {
                println!("✅ Registered tunnel '{}' ({} → {})", name, hostname, upstream);
                println!("   Next: `hyh tunnel provision {}`", name);
            }
        }
        TunnelAction::Provision { name } => {
            if !NamedTunnelManager::is_logged_in() {
                anyhow::bail!("Not logged in — run `hyh tunnel login` first");
            }
            let (tunnel_id, creds_path) =
                NamedTunnelManager::create_tunnel(&exe, &name)?;
            state
                .db
                .update_named_tunnel_credentials(&name, &tunnel_id, &creds_path)?;
            let cfg = state
                .db
                .get_named_tunnel(&name)?
                .ok_or_else(|| anyhow::anyhow!("Tunnel not found"))?;
            let config_path = state.paths.tunnel_config(&name);
            NamedTunnelManager::generate_config(
                &config_path,
                &tunnel_id,
                &creds_path,
                &[(cfg.hostname, cfg.upstream)],
            )?;
            if json {
                println!(
                    "{}",
                    json!({
                        "ok": true,
                        "tunnel_name": name,
                        "tunnel_id": tunnel_id,
                        "credentials_path": creds_path,
                        "config_path": config_path.to_string_lossy(),
                    })
                );
            } else {
                println!("✅ Provisioned tunnel '{}'", name);
                println!("   id: {}", tunnel_id);
                println!("   config: {}", config_path.display());
                println!("\n   Next: route DNS then `hyh tunnel run {}`", name);
            }
        }
        TunnelAction::Run { name } => {
            let cfg = state
                .db
                .get_named_tunnel(&name)?
                .ok_or_else(|| anyhow::anyhow!("Tunnel not found: {}", name))?;
            let tunnel_id = cfg
                .tunnel_id
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Not provisioned — run `hyh tunnel provision {}`", name))?;
            let creds = cfg
                .credentials_path
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Credentials missing — re-provision"))?;
            let config_path = state.paths.tunnel_config(&name);
            NamedTunnelManager::generate_config(
                &config_path,
                tunnel_id,
                creds,
                &[(cfg.hostname.clone(), cfg.upstream.clone())],
            )?;
            if !json {
                println!("▶ Running tunnel '{}' — Ctrl-C to stop", name);
                println!("   {} → {}", cfg.hostname, cfg.upstream);
            }
            // Foreground exec — inherits stdio so user sees cloudflared output
            let status = std::process::Command::new(&exe)
                .args(["tunnel", "--config"])
                .arg(&config_path)
                .arg("run")
                .status()?;
            if !status.success() {
                anyhow::bail!("cloudflared exited with status {}", status);
            }
            out_ok(json, "tunnel", "stopped");
        }
        TunnelAction::Remove { name } => {
            let config_path = state.paths.tunnel_config(&name);
            let _ = std::fs::remove_file(config_path);
            state.db.remove_named_tunnel(&name)?;
            if json {
                println!("{}", json!({ "ok": true, "removed": name }));
            } else {
                println!("✅ Removed tunnel '{}'", name);
                println!("   (Note: tunnel still exists on Cloudflare — delete with `cloudflared tunnel delete {}`)", name);
            }
        }
    }
    Ok(())
}

fn project_path(opt: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    let p = match opt {
        Some(p) => p,
        None => std::env::current_dir()?,
    };
    if !p.is_dir() {
        anyhow::bail!("Not a directory: {}", p.display());
    }
    Ok(p)
}

fn print_compose_output(action: &str, out: &str, json: bool) {
    if json {
        println!("{}", json!({ "ok": true, "action": action, "output": out }));
    } else {
        if !out.trim().is_empty() {
            println!("{}", out);
        }
        println!("✅ docker compose {}", action);
    }
}

fn read_stdin() -> anyhow::Result<String> {
    use std::io::Read;
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

// ──────────────────────────── helpers ────────────────────────────────────

fn rebuild_nginx(state: &AppState) -> anyhow::Result<()> {
    let all = state.db.list_domains()?;
    let nginx_conf = hyperhost_lib::nginx::config::generate(
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

fn out_ok(json: bool, subject: &str, verb: &str) {
    if json {
        println!("{}", json!({ "ok": true, "subject": subject, "action": verb }));
    } else {
        println!("✅ {} {}", subject, verb);
    }
}

fn mark(ok: bool) -> &'static str {
    if ok { "✓" } else { "✗" }
}

fn hosts_file_has(domain: &str) -> bool {
    let path = hosts_path();
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    content
        .lines()
        .any(|l| !l.trim_start().starts_with('#') && l.split_whitespace().any(|t| t == domain))
}

fn hosts_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(r"C:\Windows\System32\drivers\etc\hosts")
    }
    #[cfg(not(target_os = "windows"))]
    {
        PathBuf::from("/etc/hosts")
    }
}

fn upstream_reachable(upstream: &str) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};
    use std::time::Duration;

    // Extract host:port from URL
    let rest = upstream.trim_start_matches("http://").trim_start_matches("https://");
    let hostport = rest.split('/').next().unwrap_or(rest);
    let host_only = hostport.split(':').next().unwrap_or("127.0.0.1");
    let port = if let Some((_, p)) = hostport.split_once(':') {
        p.parse::<u16>().ok()
    } else if upstream.starts_with("https://") {
        Some(443)
    } else {
        Some(80)
    };
    let Some(port) = port else { return false };

    let addrs = match (host_only, port).to_socket_addrs() {
        Ok(a) => a,
        Err(_) => return false,
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
