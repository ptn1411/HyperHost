import { invoke } from "@tauri-apps/api/core";

export interface NamedTunnelConfig {
  id?: number;
  tunnel_name: string;
  tunnel_id?: string;
  credentials_path?: string;
  hostname: string;
  upstream: string;
  enabled: boolean;
  created_at?: string;
}

export interface DomainConfig {
  id?: number;
  domain: string;
  upstream: string;
  enabled: boolean;
  cors_enabled: boolean;
  cert_expiry?: string;
  created_at?: string;
  advanced_config?: string;
  project_path?: string;
  run_command?: string;
}

export interface DomainStatus {
  config: DomainConfig;
  cert_valid: boolean;
  cert_expiry: string | null;
}

export interface CaStatus {
  installed: boolean;
  fingerprint: string | null;
}

export interface NginxInfo {
  running: boolean;
}

export interface AppSettings {
  autostart: boolean;
  start_hidden: boolean;
}

export interface Template {
  id: string;
  name: string;
  category: string;
  default_upstream: string;
  description: string;
  advanced_config: string | null;
}

export interface PortInfo {
  port: number;
  guess: string | null;
  pid: number | null;
  process: string | null;
}

export interface ProjectInfo {
  path: string;
  name: string;
  kind: string;
  suggested_port: number;
  suggested_domain: string;
  suggested_command: string | null;
}

export interface DockerStatus {
  installed: boolean;
  version: string | null;
  daemon_running: boolean;
}

export interface ComposeService {
  name: string;
  image: string;
  state: string;
  status: string;
  ports: string;
}

export interface ComposeFileEntry {
  path: string;
  name: string;
  services: ComposeService[];
}

export interface ComposeStatus {
  files: ComposeFileEntry[];
}

export const api = {
  listDomains: () => invoke<DomainStatus[]>("list_domains"),
  addDomain: (domain: string, upstream: string, advancedConfig?: string, projectPath?: string, runCommand?: string) =>
    invoke<DomainStatus>("add_domain", { domain, upstream, advancedConfig, projectPath, runCommand }),
  editDomain: (oldDomain: string, domain: string, upstream: string, advancedConfig?: string, projectPath?: string, runCommand?: string) =>
    invoke<DomainStatus>("edit_domain", { oldDomain, domain, upstream, advancedConfig, projectPath, runCommand }),
  removeDomain: (domain: string) =>
    invoke<void>("remove_domain", { domain }),
  toggleDomain: (domain: string) =>
    invoke<boolean>("toggle_domain", { domain }),
  nginxStatus: () => invoke<NginxInfo>("nginx_status"),
  nginxStart: () => invoke<void>("nginx_start"),
  nginxStop: () => invoke<void>("nginx_stop"),
  installCa: () => invoke<void>("install_ca"),
  caStatus: () => invoke<CaStatus>("ca_status"),
  getNginxLog: (lines: number) =>
    invoke<string[]>("get_nginx_log", { lines }),
  startTunnel: (domain: string) =>
    invoke<void>("start_tunnel", { domain }),
  stopTunnel: (domain: string) =>
    invoke<void>("stop_tunnel", { domain }),


  // Named Tunnel (fixed domain via Cloudflare)
  cloudflareLogin: () => invoke<void>("cloudflare_login"),
  cloudflareLoginStatus: () => invoke<boolean>("cloudflare_login_status"),
  listNamedTunnels: () => invoke<NamedTunnelConfig[]>("list_named_tunnels"),
  addNamedTunnel: (tunnelName: string, hostname: string, upstream: string) =>
    invoke<NamedTunnelConfig>("add_named_tunnel", { tunnelName, hostname, upstream }),
  provisionNamedTunnel: (tunnelName: string) =>
    invoke<void>("provision_named_tunnel", { tunnelName }),
  startNamedTunnel: (tunnelName: string) =>
    invoke<void>("start_named_tunnel", { tunnelName }),
  stopNamedTunnel: (tunnelName: string) =>
    invoke<void>("stop_named_tunnel", { tunnelName }),
  namedTunnelRunning: (tunnelName: string) =>
    invoke<boolean>("named_tunnel_running", { tunnelName }),
  removeNamedTunnel: (tunnelName: string) =>
    invoke<void>("remove_named_tunnel", { tunnelName }),

  // CORS / Export / Import
  toggleCors: (domain: string) =>
    invoke<boolean>("toggle_cors", { domain }),
  exportDomains: () =>
    invoke<string>("export_domains"),
  importDomains: (json: string) =>
    invoke<DomainStatus[]>("import_domains", { json }),

  // App Settings
  getAppSettings: () => invoke<AppSettings>("get_app_settings"),
  setAutostart: (enabled: boolean) => invoke<void>("set_autostart", { enabled }),
  setStartHidden: (enabled: boolean) => invoke<void>("set_start_hidden", { enabled }),

  // Quick Start (templates / ports / projects)
  listTemplates: () => invoke<Template[]>("list_templates"),
  scanPorts: () => invoke<PortInfo[]>("scan_ports"),
  scanProjects: (root: string, depth?: number) =>
    invoke<ProjectInfo[]>("scan_projects", { root, depth }),
  getHomeDir: () => invoke<string>("get_home_dir"),
  openTerminal: (path: string, command?: string) =>
    invoke<void>("open_terminal", { path, command }),
  openFolder: (path: string) =>
    invoke<void>("open_folder", { path }),

  // Nginx prod <-> dev conversion
  importNginxConfig: (filePath: string) =>
    invoke<ImportedNginx>("import_nginx_config", { filePath }),
  importNginxConfigText: (content: string) =>
    invoke<ImportedNginx>("import_nginx_config_text", { content }),
  validateNginxConfig: (content: string) =>
    invoke<string>("validate_nginx_config", { content }),
  exportNginxConfigToProject: (domain: string, prodDomain: string, prodUpstream: string) =>
    invoke<string>("export_nginx_config_to_project", { domain, prodDomain, prodUpstream }),

  // Docker Compose (per-project)
  dockerCheck: () => invoke<DockerStatus>("docker_check"),
  composeStatus: (projectPath: string) =>
    invoke<ComposeStatus>("compose_status", { projectPath }),
  composeUp: (projectPath: string, file?: string) =>
    invoke<string>("compose_up", { projectPath, file }),
  composeDown: (projectPath: string, file?: string) =>
    invoke<string>("compose_down", { projectPath, file }),
  composeRestart: (projectPath: string, file?: string) =>
    invoke<string>("compose_restart", { projectPath, file }),
  composeLogs: (projectPath: string, file?: string, lines?: number) =>
    invoke<string>("compose_logs", { projectPath, file, lines }),
  composeSaveFile: (projectPath: string, fileName: string, content: string) =>
    invoke<string>("compose_save_file", { projectPath, fileName, content }),
};

export interface ImportedNginx {
  advanced_config: string;
  suggested_upstream: string | null;
  server_name: string | null;
}
