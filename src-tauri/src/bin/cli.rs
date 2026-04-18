use clap::{Parser, Subcommand};
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, Color, Table,
};

#[derive(Parser)]
#[command(
    name = "hyh",
    version = "0.4.1",
    about = "⚡ HyperHost CLI — Local HTTPS domain manager",
    long_about = "Manage local virtual domains with HTTPS certificates.\nAdd domains like myapp.test that proxy to your dev server with trusted SSL.\n\nUsage: hyh add myapp.test http://127.0.0.1:3000"
)]
struct Cli {
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
}

#[derive(Subcommand)]
enum CaAction {
    /// Install CA into Windows trust store
    Install,
    /// Check CA installation status
    Status,
}

fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(false)
        .without_time()
        .init();

    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("❌ Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let state = hyperhost_lib::init_state()?;

    match cli.command {
        Commands::Add { domain, upstream } => cmd_add(&state, &domain, &upstream)?,
        Commands::Remove { domain } => cmd_remove(&state, &domain)?,
        Commands::List => cmd_list(&state)?,
        Commands::Toggle { domain } => cmd_toggle(&state, &domain)?,
        Commands::Nginx { action } => cmd_nginx(&state, action)?,
        Commands::Ca { action } => cmd_ca(&state, action)?,
    }

    Ok(())
}

fn cmd_add(
    state: &hyperhost_lib::state::AppState,
    domain: &str,
    upstream: &str,
) -> anyhow::Result<()> {
    if !domain.ends_with(".test") && !domain.ends_with(".local") {
        anyhow::bail!("Domain must end with .test or .local");
    }
    if !upstream.starts_with("http://") && !upstream.starts_with("https://") {
        anyhow::bail!("Upstream must start with http:// or https://");
    }

    println!("🔐 Issuing certificate for {}...", domain);

    // Try rcgen first, fallback to mkcert
    let cert_dir = state.paths.cert_dir();
    std::fs::create_dir_all(&cert_dir)?;

    let (cert_pem, key_pem) = match state.ca.issue_for_domain(domain) {
        Ok((cert, key)) => {
            std::fs::write(cert_dir.join(format!("{}.crt", domain)), &cert)?;
            std::fs::write(cert_dir.join(format!("{}.key", domain)), &key)?;
            println!("  ✓ Certificate issued (rcgen)");
            (cert, key)
        }
        Err(rcgen_err) => {
            println!("  ⚠ rcgen failed: {}, trying mkcert...", rcgen_err);
            let mkcert = hyperhost_lib::cert::mkcert::MkcertRunner::find()
                .ok_or_else(|| anyhow::anyhow!("rcgen failed and mkcert not found"))?;
            mkcert.issue_for_domain(domain, &cert_dir)?;
            let cert = std::fs::read_to_string(cert_dir.join(format!("{}.crt", domain)))?;
            let key = std::fs::read_to_string(cert_dir.join(format!("{}.key", domain)))?;
            println!("  ✓ Certificate issued (mkcert fallback)");
            (cert, key)
        }
    };

    // Save to DB
    let expiry = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(
            hyperhost_lib::cert::ca::CERT_VALIDITY_DAYS,
        ))
        .map(|d| d.to_rfc3339());

    let cfg = hyperhost_lib::db::DomainConfig {
        id: None,
        domain: domain.to_string(),
        upstream: upstream.to_string(),
        enabled: true,
        cors_enabled: false,
        cert_expiry: expiry,
        created_at: None,
        advanced_config: None,
        project_path: None,
        run_command: None,
    };
    state.db.upsert_domain(&cfg, &cert_pem, &key_pem)?;
    println!("📦 Saved to database");

    // Sync hosts
    let active = state.db.list_enabled_domains()?;
    hyperhost_lib::dns::hosts::sync_hosts(&active)?;
    println!("📝 Hosts file updated");

    // Rebuild nginx
    rebuild_nginx(state)?;
    println!("🔄 nginx config regenerated");

    println!("\n✅ https://{} → {}", domain, upstream);
    Ok(())
}

fn cmd_remove(state: &hyperhost_lib::state::AppState, domain: &str) -> anyhow::Result<()> {
    state.db.remove_domain(domain)?;

    let cert_dir = state.paths.cert_dir();
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.crt", domain)));
    let _ = std::fs::remove_file(cert_dir.join(format!("{}.key", domain)));

    let active = state.db.list_enabled_domains()?;
    hyperhost_lib::dns::hosts::sync_hosts(&active)?;
    rebuild_nginx(state)?;

    println!("✅ Removed {}", domain);
    Ok(())
}

fn cmd_list(state: &hyperhost_lib::state::AppState) -> anyhow::Result<()> {
    let domains = state.db.list_domains()?;

    if domains.is_empty() {
        println!("No domains configured. Use 'dh add <domain> <upstream>' to get started.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Domain")
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
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
            Cell::new(&d.domain)
                .fg(Color::White)
                .add_attribute(Attribute::Bold),
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

fn cmd_toggle(state: &hyperhost_lib::state::AppState, domain: &str) -> anyhow::Result<()> {
    let new_state = state.db.toggle_domain(domain)?;
    let active = state.db.list_enabled_domains()?;
    hyperhost_lib::dns::hosts::sync_hosts(&active)?;
    rebuild_nginx(state)?;

    let icon = if new_state {
        "🟢 enabled"
    } else {
        "⚫ disabled"
    };
    println!("✅ {} → {}", domain, icon);
    Ok(())
}

fn cmd_nginx(state: &hyperhost_lib::state::AppState, action: NginxAction) -> anyhow::Result<()> {
    match action {
        NginxAction::Start => {
            rebuild_nginx(state)?;
            state.nginx.start()?;
            println!("✅ nginx started");
        }
        NginxAction::Stop => {
            state.nginx.stop()?;
            println!("✅ nginx stopped");
        }
        NginxAction::Reload => {
            rebuild_nginx(state)?;
            state.nginx.reload()?;
            println!("✅ nginx reloaded");
        }
        NginxAction::Status => {
            let running = state.nginx.is_running();
            println!(
                "nginx: {}",
                if running {
                    "🟢 running"
                } else {
                    "⚫ stopped"
                }
            );
        }
        NginxAction::Logs { lines } => {
            let log_path = state.paths.nginx_logs().join("error.log");
            let content = std::fs::read_to_string(&log_path).unwrap_or_default();
            let all: Vec<&str> = content.lines().collect();
            let start = all.len().saturating_sub(lines);
            for line in &all[start..] {
                println!("{}", line);
            }
        }
    }
    Ok(())
}

fn cmd_ca(state: &hyperhost_lib::state::AppState, action: CaAction) -> anyhow::Result<()> {
    let ca_cert = state.paths.ca_cert();
    match action {
        CaAction::Install => {
            let result = {
                #[cfg(target_os = "windows")]
                {
                    hyperhost_lib::cert::windows_store::install_ca(&ca_cert)
                }
                #[cfg(target_os = "macos")]
                {
                    hyperhost_lib::cert::macos_store::install_ca(&ca_cert)
                }
                #[cfg(target_os = "linux")]
                {
                    hyperhost_lib::cert::linux_store::install_ca(&ca_cert)
                }
                #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
                {
                    Err(anyhow::anyhow!(
                        "CA installation not supported on this platform"
                    ))
                }
            };
            result?;

            if let Some(mkcert) = hyperhost_lib::cert::mkcert::MkcertRunner::find() {
                match mkcert.install_ca() {
                    Ok(_) => println!("  ✓ mkcert: Firefox NSS trusted"),
                    Err(e) => println!("  ⚠ mkcert -install failed: {}", e),
                }
            }

            println!("\n✅ CA installed successfully");
        }
        CaAction::Status => {
            let installed = {
                #[cfg(target_os = "windows")]
                {
                    hyperhost_lib::cert::windows_store::is_ca_installed(&ca_cert)
                }
                #[cfg(target_os = "macos")]
                {
                    hyperhost_lib::cert::macos_store::is_ca_installed(&ca_cert)
                }
                #[cfg(target_os = "linux")]
                {
                    hyperhost_lib::cert::linux_store::is_ca_installed(&ca_cert)
                }
                #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
                {
                    false
                }
            };
            println!(
                "CA: {}",
                if installed {
                    "🟢 installed & trusted"
                } else {
                    "⚫ not installed"
                }
            );
            if let Some(fp) = state.ca.fingerprint() {
                println!("SHA-256: {}", fp);
            }
        }
    }
    Ok(())
}

fn rebuild_nginx(state: &hyperhost_lib::state::AppState) -> anyhow::Result<()> {
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
