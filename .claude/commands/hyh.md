---
description: "Manage local HTTPS domains via HyperHost. Trigger on: 'set up domain', 'add domain', 'local HTTPS', 'hyh', 'hyperhost', 'dev domain', 'proxy my app', 'debug domain', 'fix hosts', 'nginx issue', 'scan ports'."
---

# HyperHost Domain Manager

HyperHost manages local HTTPS dev domains. You can interact with it two ways:
1. **MCP tools** (prefixed `mcp__hyperhost__`) — structured data, preferred when MCP server is connected
2. **`hyh` CLI** — run shell commands directly, works anywhere, supports `--json` for machine output

Use MCP tools when available. Fall back to `hyh` CLI if MCP is not connected or for operations MCP doesn't cover.

## hyh CLI Reference

```
hyh [--json] <COMMAND>
```

### Domain Management
- `hyh add <domain> <upstream>` — Add domain with auto HTTPS cert
- `hyh remove <domain>` — Delete domain + cert
- `hyh list [--json]` — List all configured domains
- `hyh toggle <domain>` — Enable/disable a domain
- `hyh cors <domain>` — Toggle CORS headers
- `hyh open <domain>` — Open https://<domain> in browser
- `hyh doctor <domain>` — Health checks (cert, hosts, upstream, nginx)
- `hyh export` — Export all domain configs to JSON
- `hyh import <file>` — Import domains from JSON file

### Nginx Proxy
- `hyh nginx start` — Start nginx proxy
- `hyh nginx stop` — Stop nginx proxy
- `hyh nginx reload` — Reload config
- `hyh nginx status` — Check if running
- `hyh nginx logs [-n <lines>]` — Show error logs
- `hyh nginx import <file>` — Convert production nginx config to dev snippet
- `hyh nginx validate <file>` — Validate dev nginx snippet
- `hyh nginx export <domain> <prod_domain> <prod_upstream>` — Generate production config

### Certificate Authority
- `hyh ca install` — Install CA into system trust store (UAC prompt on Windows)
- `hyh ca status` — Check CA installation + fingerprint

### Scanning & Detection
- `hyh scan ports` — List listening TCP ports with PID/process/framework guess
- `hyh scan projects [--root <dir>] [--depth N]` — Find dev projects

### Docker Compose
- `hyh docker check` — Check Docker CLI + daemon
- `hyh docker status [--project <dir>]` — List compose files + services
- `hyh docker up/down/restart [--project <dir>] [--file <name>]`
- `hyh docker logs [--project <dir>] [-n <lines>]`
- `hyh docker save <name> [--project <dir>] [--content <yaml>]`

### Cloudflare Tunnels
- `hyh tunnel login` — Authenticate with Cloudflare
- `hyh tunnel status` / `hyh tunnel list`
- `hyh tunnel add <name> <hostname> <upstream>`
- `hyh tunnel provision <name>` — Create tunnel on Cloudflare
- `hyh tunnel run <name>` — Run tunnel (foreground)
- `hyh tunnel remove <name>`

### AI CLI Integration
- `hyh skill init` — Auto-register MCP server + skill into Claude Code, Gemini CLI, Codex CLI
- `hyh mcp serve` — Start MCP server on stdio
- `hyh mcp snippet` — Print MCP client config snippet

## MCP Tools (when connected)

### Read-only
- `list_domains` / `nginx_status` / `nginx_logs(lines?)` / `scan_ports`
- `scan_projects(root?, depth?)` / `doctor(domain)` / `ca_status` / `elevation_status`
- `export_domains` / `docker_check` / `compose_status(project_path)`
- `list_named_tunnels`

### Write
- `add_domain(domain, upstream)` / `edit_domain(domain, upstream, advanced_config?)`
- `remove_domain(domain)` / `toggle_domain(domain)` / `toggle_cors(domain)`
- `import_domains(json_data)` / `ca_install`
- `nginx_start` / `nginx_stop` / `nginx_reload` / `open_domain(domain)`
- `compose_up/down/restart/logs(project_path, file?)` / `compose_save_file(...)`
- `add_named_tunnel(name, hostname, upstream)`
- `install_project_skills(project_path)` — install AI skill files into a project directory

## Workflows

### Quick Setup
1. `scan_ports` (or `hyh scan ports`) to find running dev servers
2. `add_domain` (or `hyh add myapp.test http://127.0.0.1:3000`)
3. `nginx_start` (or `hyh nginx start`) if not running
4. `ca_status` — if not installed, run `ca_install` (or `hyh ca install`)
5. `doctor` (or `hyh doctor myapp.test`) to verify
6. `open_domain` (or `hyh open myapp.test`)

### Diagnose Issues
1. `hyh doctor <domain>` — check all components
2. `hyh nginx logs` — check proxy errors
3. `hyh scan ports` — verify upstream is listening
4. `hyh nginx reload` — reload after config changes

### Batch Operations with CLI
```bash
hyh --json list                    # machine-readable domain list
hyh --json scan ports              # JSON port scan
hyh export > backup.json           # backup all domains
hyh import backup.json             # restore domains
```

## Rules
- Domains MUST end with `.test` or `.local`
- Upstream MUST start with `http://` or `https://`
- Always verify with `doctor` after adding a domain
- App runs as normal user; hosts file writes trigger UAC prompt
- CA install is one-time — check `ca_status` first
