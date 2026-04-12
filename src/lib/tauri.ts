import { invoke } from "@tauri-apps/api/core";

export interface DomainConfig {
  id?: number;
  domain: string;
  upstream: string;
  enabled: boolean;
  cors_enabled: boolean;
  cert_expiry?: string;
  created_at?: string;
  advanced_config?: string;
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

export const api = {
  listDomains: () => invoke<DomainStatus[]>("list_domains"),
  addDomain: (domain: string, upstream: string, advancedConfig?: string) =>
    invoke<DomainStatus>("add_domain", { domain, upstream, advancedConfig }),
  editDomain: (oldDomain: string, domain: string, upstream: string, advancedConfig?: string) =>
    invoke<DomainStatus>("edit_domain", { oldDomain, domain, upstream, advancedConfig }),
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
  toggleCors: (domain: string) =>
    invoke<boolean>("toggle_cors", { domain }),
  exportDomains: () =>
    invoke<string>("export_domains"),
  importDomains: (json: string) =>
    invoke<DomainStatus[]>("import_domains", { json }),
};
