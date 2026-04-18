# HyperHost CLI (`hyh`) Reference Guide

The `hyh` command-line tool provides full control over your HyperHost environment, allowing you to manage virtual domains, Nginx configuration, Docker containers, Cloudflare testing tunnels, and the new Model Context Protocol (MCP) server.

> **Note:** Some commands (like managing Nginx or the `hosts` file) require `hyh` to be run with Administrator or `sudo` privileges.

---

## ⚡ Core Domain Management

| Command | Description | Example |
|---|---|---|
| `hyh add <domain> <upstream>` | Creates a local virtual domain tied to an upstream and generates a trusted HTTPS cert. | `hyh add api.test http://127.0.0.1:3000` |
| `hyh remove <domain>` | Deletes the domain, removes it from Nginx, and deletes its certificates. | `hyh remove api.test` |
| `hyh list` | Lists all configured domains in a formatted table. | `hyh list` |
| `hyh toggle <domain>` | Enables or disables a domain temporarily without deleting its configuration. | `hyh toggle api.test` |
| `hyh cors <domain>` | Toggles wildcard CORS headers for the given domain. | `hyh cors api.test` |
| `hyh open <domain>` | Opens `https://<domain>` directly in your default web browser. | `hyh open api.test` |
| `hyh doctor <domain>` | Runs diagnostic health checks (Cert validity, hosts mapping, upstream reachability). | `hyh doctor api.test` |

---

## 🔄 Configuration Import / Export

| Command | Description | Example |
|---|---|---|
| `hyh export` | Outputs the current database of domains in JSON format to `stdout`. | `hyh export > domains.json` |
| `hyh import <file>` | Restores domains from a JSON export payload. | `hyh import domains.json` |

---

## 🐳 Docker Integration (New)

The `hyh docker` command provides seamless bridging between your local docker-compose environments and HyperHost. 

| Command | Description | Example |
|---|---|---|
| `hyh docker check` | Checks if the Docker daemon is accessible globally. | `hyh docker check` |
| `hyh docker status [--project <dir>]` | Shows docker-compose state in a project directory. | `hyh docker status` |
| `hyh docker up [--project <dir>] [--file <file>]` | Safely runs `docker compose up -d` for the given project path. | `hyh docker up` |
| `hyh docker down [--project <dir>] [--file <file>]`| Safely tears down the docker compose cluster. | `hyh docker down` |
| `hyh docker restart [--project <dir>] [--file <file>]`| Restarts the compose cluster for the project. | `hyh docker restart` |
| `hyh docker logs [--lines 200]` | Streams or outputs recent docker compose logs. | `hyh docker logs --lines 50` |
| `hyh docker save <name> [--content <string>]` | Saves a template compose file into the current project dynamically. | `hyh docker save docker-compose.yml` |

---

## 🌀 Cloudflare Named Tunnels (New)

| Command | Description | Example |
|---|---|---|
| `hyh tunnel login` | Opens browser to auth Cloudflare (creates `cert.pem`). | `hyh tunnel login` |
| `hyh tunnel status` | Checks login state and lists status of configured tunnels. | `hyh tunnel status` |
| `hyh tunnel add <name> <host> <upstream>` | Registers a persistent Cloudflare tracking ID to a specific URL. | `hyh tunnel add dev api.example.com http://localhost:80` |
| `hyh tunnel provision <name>` | Provisions the tunnel configuration against the Cloudflare API. | `hyh tunnel provision dev` |
| `hyh tunnel run <name>` | Initiates `cloudflared` to broadcast the tunnel locally. | `hyh tunnel run dev` |
| `hyh tunnel remove <name>` | Removes a registered named tunnel from local storage. | `hyh tunnel remove dev` |

---

## 🤖 Model Context Protocol (MCP) Server (New)

HyperHost allows LLMs running via MCP (such as Claude Code) to interact with the backend infrastructure natively.

| Command | Description | Example |
|---|---|---|
| `hyh mcp serve` | Spawns the MCP JSON-RPC interface via `stdio`. | `hyh mcp serve` |
| `hyh mcp snippet` | Prints configuration required to embed `hyh` into Cursor or Claude Code. | `hyh mcp snippet` |

---

## 🔍 Scanning Tools (New)

| Command | Description | Example |
|---|---|---|
| `hyh scan ports` | Lists listening TCP ports with their PID and process name. | `hyh scan ports` |
| `hyh scan projects [--root <dir>] [--depth 3]` | Traverses directories searching for supported frameworks/languages. | `hyh scan projects --depth 5` |

---

## 🚦 Nginx & CA Administration

| Command | Description | Example |
|---|---|---|
| `hyh nginx start / stop / reload / status` | Basic lifecycle tools for the bundled Nginx service. | `hyh nginx reload` |
| `hyh nginx logs [--lines 20]` | Prints out recent `error.log` output. | `hyh nginx logs --lines 100` |
| `hyh nginx validate <file>` | Uses `nginx -t` to validate snippets prior to application. | `hyh nginx validate .conf` |
| `hyh nginx import <file>` | Strip production logic and auto-mount to dev Nginx configuration. | `hyh nginx import prod.conf` |
| `hyh ca install / status` | Inspect or inject the localized HTTPS Authority into Windows/macOS. | `hyh ca install` |

---
*Tip: Passing `--json` to most `hyh` commands will output results natively formatted as JSON payloads for automated scripting.*
