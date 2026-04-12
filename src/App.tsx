import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import { NamedTunnelPanel } from "./components/NamedTunnelPanel";
import { NginxEditorMode } from "./components/NginxEditorMode";
import { TrafficInspector } from "./components/TrafficInspector";
import { UpdateDialog } from "./components/UpdateDialog";
import { api, CaStatus, DomainStatus, NginxInfo } from "./lib/tauri";

function App() {
  const [domains, setDomains] = useState<DomainStatus[]>([]);
  const [caStatus, setCaStatus] = useState<CaStatus | null>(null);
  const [nginxInfo, setNginxInfo] = useState<NginxInfo | null>(null);
  const [domain, setDomain] = useState("");
  const [upstream, setUpstream] = useState("http://127.0.0.1:3000");
  const [editorMode, setEditorMode] = useState<"hidden" | "add" | "edit">(
    "hidden",
  );
  const [editingData, setEditingData] = useState<{
    domain: string;
    upstream: string;
    advancedConfig: string;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [logs, setLogs] = useState<string[]>([]);
  const [showLogs, setShowLogs] = useState(false);
  const [activeTab, setActiveTab] = useState<"domains" | "traffic" | "named-tunnel">("domains");
  const [tunnels, setTunnels] = useState<
    Record<string, { url: string; loading: boolean }>
  >({});
  const [deleteConfirm, setDeleteConfirm] = useState<string | null>(null);
  const [stats, setStats] = useState<Record<string, { count: number; totalMs: number }>>({});

  const refresh = async () => {
    try {
      const [d, ca, nx] = await Promise.all([
        api.listDomains(),
        api.caStatus(),
        api.nginxStatus(),
      ]);
      setDomains(d);
      setCaStatus(ca);
      setNginxInfo(nx);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    refresh();
    const unlistenReady = listen<{ domain: string; url: string }>(
      "tunnel_ready",
      (event) => {
        setTunnels((prev) => ({
          ...prev,
          [event.payload.domain]: { url: event.payload.url, loading: false },
        }));
      },
    );
    const unlistenError = listen<{ domain: string; error: string }>(
      "tunnel_error",
      (event) => {
        setTunnels((prev) => {
          const next = { ...prev };
          delete next[event.payload.domain];
          return next;
        });
        setError(`Tunnel [${event.payload.domain}]: ${event.payload.error}`);
      },
    );
    const unlistenTraffic = listen<{ host: string; latency: string }>(
      "nginx_access_log",
      (event) => {
        const host = event.payload.host;
        const ms = parseFloat(event.payload.latency) * 1000;
        if (!host || isNaN(ms)) return;
        setStats((prev) => {
          const cur = prev[host] ?? { count: 0, totalMs: 0 };
          return { ...prev, [host]: { count: cur.count + 1, totalMs: cur.totalMs + ms } };
        });
      },
    );
    return () => {
      unlistenReady.then((fn) => fn());
      unlistenError.then((fn) => fn());
      unlistenTraffic.then((fn) => fn());
    };
  }, []);

  const handleSaveDomain = async (
    parsedDomain: string,
    parsedUpstream: string,
    parsedAdvanced: string,
  ) => {
    setLoading(true);
    setError(null);
    try {
      if (editorMode === "edit" && editingData) {
        // Edit mode: pass old domain name for proper rename handling
        await api.editDomain(
          editingData.domain,
          parsedDomain.trim(),
          parsedUpstream.trim(),
          parsedAdvanced,
        );
      } else {
        // Add mode
        await api.addDomain(
          parsedDomain.trim(),
          parsedUpstream.trim(),
          parsedAdvanced,
        );
      }
      setEditorMode("hidden");
      setEditingData(null);
      setDomain("");
      setUpstream("http://127.0.0.1:3000");
      await refresh();
    } catch (err: any) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleQuickAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!domain.trim()) return;
    await handleSaveDomain(domain, upstream, "");
  };

  const handleEdit = (d: DomainStatus) => {
    setEditingData({
      domain: d.config.domain,
      upstream: d.config.upstream,
      advancedConfig: d.config.advanced_config || "",
    });
    setEditorMode("edit");
    window.scrollTo({ top: 0, behavior: "smooth" });
  };

  const handleRemove = (d: string) => {
    setDeleteConfirm(d);
  };

  const confirmRemove = async () => {
    if (!deleteConfirm) return;
    try {
      await api.removeDomain(deleteConfirm);
      await refresh();
    } catch (err: any) {
      setError(String(err));
    } finally {
      setDeleteConfirm(null);
    }
  };

  const handleToggle = async (d: string) => {
    try {
      await api.toggleDomain(d);
      await refresh();
    } catch (err: any) {
      setError(String(err));
    }
  };

  const handleInstallCa = async () => {
    setLoading(true);
    try {
      await api.installCa();
      await refresh();
    } catch (err: any) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const handleNginxToggle = async () => {
    setLoading(true);
    try {
      if (nginxInfo?.running) {
        await api.nginxStop();
      } else {
        await api.nginxStart();
      }
      await refresh();
    } catch (err: any) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  const toggleTunnel = async (domain: string) => {
    if (tunnels[domain] && !tunnels[domain].loading) {
      setTunnels((prev) => {
        const next = { ...prev };
        delete next[domain];
        return next;
      });
      await api.stopTunnel(domain);
    } else {
      setTunnels((prev) => ({ ...prev, [domain]: { url: "", loading: true } }));
      await api.startTunnel(domain);
    }
  };

  const handleToggleCors = async (domain: string) => {
    try {
      await api.toggleCors(domain);
      await refresh();
    } catch (err: any) {
      setError(String(err));
    }
  };

  const handleExport = async () => {
    try {
      const json = await api.exportDomains();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `hyperhost-domains-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (err: any) {
      setError(String(err));
    }
  };

  const handleImport = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const imported = await api.importDomains(text);
        await refresh();
        if (imported.length === 0) {
          setError("No valid domains found in import file.");
        }
      } catch (err: any) {
        setError(String(err));
      }
    };
    input.click();
  };

  const handleShowLogs = async () => {
    try {
      const l = await api.getNginxLog(50);
      setLogs(l);
      setShowLogs(!showLogs);
    } catch (err: any) {
      setError(String(err));
    }
  };

  return (
    <div className="min-h-screen bg-surface p-6 font-sans">
      <UpdateDialog />
      <div className="max-w-5xl mx-auto">
        {/* Header */}
        <header className="flex items-center justify-between mb-10 pb-6 border-b border-surface-3/30">
          <div>
            <h1 className="text-3xl font-bold text-text flex items-center gap-3 tracking-tight font-mono">
              <span className="text-accent bg-accent/10 p-2 rounded-xl">
                <svg
                  className="w-6 h-6"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M13 10V3L4 14h7v7l9-11h-7z"
                  />
                </svg>
              </span>
              HyperHost Manager
            </h1>
            <p className="text-text-muted mt-2 text-sm">
              Local HTTPS domains for development •{" "}
              <span className="font-mono text-xs">v0.1.9</span>
            </p>
          </div>
          <div className="flex items-center gap-3">
            {/* Nginx Status */}
            <button
              onClick={handleNginxToggle}
              disabled={loading}
              className={`
                flex items-center gap-2.5 px-5 py-2.5 rounded-lg text-sm font-semibold cursor-pointer
                transition-all duration-200 border-2
                ${
                  nginxInfo?.running
                    ? "bg-success/10 text-success border-success/20 hover:bg-success/20 hover:border-success/40"
                    : "bg-surface-2 text-text-muted border-surface-3 hover:border-text-muted/50 hover:text-text"
                }
              `}>
              <span
                className={`w-2.5 h-2.5 rounded-full shadow-sm ${nginxInfo?.running ? "bg-success shadow-success/50 animate-pulse" : "bg-danger"}`}
              />
              <span className="font-mono">
                {nginxInfo?.running ? "nginx: RUNNING" : "nginx: STOPPED"}
              </span>
            </button>

            {/* CA Status */}
            {caStatus && !caStatus.installed && (
              <button
                onClick={handleInstallCa}
                disabled={loading}
                className="flex items-center gap-2 px-5 py-2.5 rounded-lg text-sm font-semibold cursor-pointer bg-warning/10 text-warning border-2 border-warning/20 hover:bg-warning/20 hover:border-warning/40 transition-all duration-200">
                <svg
                  className="w-4 h-4"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"
                  />
                </svg>
                Install CA
              </button>
            )}
            {caStatus?.installed && (
              <div
                className="flex items-center gap-2 px-4 py-2.5 rounded-lg text-sm font-medium bg-surface-2 border border-surface-3 text-text-muted"
                title={caStatus.fingerprint ? `SHA-256: ${caStatus.fingerprint}` : undefined}>
                <span className="text-success">
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2.5}
                      d="M5 13l4 4L19 7"
                    />
                  </svg>
                </span>
                CA Trusted
                {caStatus.fingerprint && (
                  <span className="font-mono text-xs text-text-muted/60 truncate max-w-[120px]">
                    {caStatus.fingerprint.slice(0, 11)}…
                  </span>
                )}
              </div>
            )}
          </div>
        </header>

        {/* Tabs */}
        <div className="flex items-center gap-1 mb-8 p-1 bg-surface-2 rounded-xl border border-surface-3/50 w-fit">
          <button
            onClick={() => setActiveTab("domains")}
            className={`px-5 py-2 rounded-lg text-sm font-semibold transition-all ${activeTab === "domains" ? "bg-surface shadow-sm text-text" : "text-text-muted hover:text-text cursor-pointer"}`}>
            Tên Miền & Proxy
          </button>
          <button
            onClick={() => setActiveTab("traffic")}
            className={`px-5 py-2 rounded-lg text-sm font-semibold transition-all ${activeTab === "traffic" ? "bg-surface shadow-sm text-text" : "text-text-muted hover:text-text cursor-pointer"}`}>
            Live Traffic
          </button>
          <button
            onClick={() => setActiveTab("named-tunnel")}
            className={`px-5 py-2 rounded-lg text-sm font-semibold transition-all ${activeTab === "named-tunnel" ? "bg-surface shadow-sm text-text" : "text-text-muted hover:text-text cursor-pointer"}`}>
            Named Tunnel
          </button>
        </div>

        {/* Error Banner */}
        {error && (
          <div className="mb-8 p-4 rounded-xl bg-danger/10 border border-danger/30 text-danger text-sm flex items-start gap-3 shadow-sm">
            <svg
              className="w-5 h-5 shrink-0 mt-0.5"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <span className="flex-1 font-mono text-xs mt-0.5">{error}</span>
            <button
              onClick={() => setError(null)}
              className="text-danger hover:text-danger/70 cursor-pointer p-1 rounded-md hover:bg-danger/20 transition-colors">
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24">
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
        )}

        {activeTab === "traffic" ? (
          <TrafficInspector />
        ) : activeTab === "named-tunnel" ? (
          <NamedTunnelPanel />
        ) : (
          <>
            {editorMode !== "hidden" ? (
              <NginxEditorMode
                isEditing={editorMode === "edit"}
                initialDomain={
                  editorMode === "edit"
                    ? editingData?.domain
                    : editorMode === "add"
                      ? domain
                      : ""
                }
                initialUpstream={
                  editorMode === "edit"
                    ? editingData?.upstream
                    : editorMode === "add"
                      ? upstream
                      : "http://127.0.0.1:3000"
                }
                initialAdvancedConfig={
                  editorMode === "edit" ? editingData?.advancedConfig : ""
                }
                loading={loading}
                onSave={handleSaveDomain}
                onCancel={() => setEditorMode("hidden")}
              />
            ) : (
              <form
                onSubmit={handleQuickAdd}
                className="mb-10 p-6 rounded-xl bg-surface-2 shadow-md border border-surface-3/30 transition-all duration-200">
                <h2 className="text-lg font-semibold mb-5 text-text flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    Thêm Route Nhanh
                  </div>
                  <button
                    type="button"
                    onClick={() => setEditorMode("add")}
                    className="text-xs font-semibold px-3 py-1.5 rounded bg-surface-3 hover:bg-accent/20 hover:text-accent text-text-muted transition-colors flex items-center gap-1.5">
                    <svg
                      className="w-3.5 h-3.5"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24">
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4"
                      />
                    </svg>
                    Chế độ Code Editor
                  </button>
                </h2>
                <div className="flex flex-col md:flex-row gap-4">
                  <div className="flex-1">
                    <label className="block text-xs font-semibold tracking-wide text-text-muted mb-2 uppercase">
                      Local Domain
                    </label>
                    <div className="relative">
                      <span className="absolute left-3.5 top-3 text-text-muted">
                        <svg
                          className="w-4 h-4"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"
                          />
                        </svg>
                      </span>
                      <input
                        type="text"
                        value={domain}
                        onChange={(e) => setDomain(e.target.value)}
                        placeholder="myapp.test"
                        className="w-full pl-10 pr-4 py-2.5 rounded-lg bg-surface border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 transition-all"
                      />
                    </div>
                  </div>
                  <div className="flex-1">
                    <label className="block text-xs font-semibold tracking-wide text-text-muted mb-2 uppercase">
                      Upstream Server
                    </label>
                    <div className="relative">
                      <span className="absolute left-3.5 top-3 text-text-muted">
                        <svg
                          className="w-4 h-4"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01"
                          />
                        </svg>
                      </span>
                      <input
                        type="text"
                        value={upstream}
                        onChange={(e) => setUpstream(e.target.value)}
                        placeholder="http://127.0.0.1:3000"
                        className="w-full pl-10 pr-4 py-2.5 rounded-lg bg-surface border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 transition-all"
                      />
                    </div>
                  </div>
                  <div className="flex items-end">
                    <button
                      type="submit"
                      disabled={loading || !domain.trim()}
                      className="w-full md:w-auto px-8 py-2.5 rounded-lg bg-accent text-white font-semibold cursor-pointer hover:bg-accent-hover disabled:opacity-50 disabled:cursor-not-allowed transition-all duration-200 active:scale-95 shadow-md shadow-accent/20">
                      {loading ? "..." : "Tạo Nhanh"}
                    </button>
                  </div>
                </div>
              </form>
            )}

            {/* Domain List */}
            <div className="space-y-4">
              <div className="flex items-end justify-between mb-4 border-b border-surface-3/30 pb-3">
                <h2 className="text-xl font-bold text-text flex items-center gap-2">
                  Active Routes
                  <span className="bg-surface-3 text-text text-xs px-2.5 py-0.5 rounded-full font-mono">
                    {domains.length}
                  </span>
                </h2>
                <div className="flex items-center gap-2">
                  <button
                    onClick={handleImport}
                    className="text-sm font-medium text-text-muted hover:text-accent transition-colors cursor-pointer flex items-center gap-1.5"
                    title="Import domains from JSON">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
                    </svg>
                    Import
                  </button>
                  <span className="text-surface-3">|</span>
                  <button
                    onClick={handleExport}
                    className="text-sm font-medium text-text-muted hover:text-accent transition-colors cursor-pointer flex items-center gap-1.5"
                    title="Export all domains to JSON">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                    </svg>
                    Export
                  </button>
                  <span className="text-surface-3">|</span>
                  <button
                    onClick={handleShowLogs}
                    className="text-sm font-medium text-text-muted hover:text-accent transition-colors cursor-pointer flex items-center gap-1.5">
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    {showLogs ? "Hide Error Log" : "Nginx Error Log"}
                  </button>
                </div>
              </div>

              {domains.length === 0 && (
                <div className="text-center py-20 bg-surface-2 border border-surface-3/30 rounded-xl border-dashed">
                  <svg
                    className="w-12 h-12 mx-auto mb-4 text-text-muted/30"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24">
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={1.5}
                      d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  <p className="text-text-muted font-medium">
                    No domains configured
                  </p>
                  <p className="text-text-muted/60 text-sm mt-1">
                    Add your first local proxy route above.
                  </p>
                </div>
              )}

              {domains.map((d) => (
                <div
                  key={d.config.domain}
                  className={`
                group p-5 rounded-xl border transition-all duration-300
                ${
                  d.config.enabled
                    ? "bg-surface-2 border-surface-3/50 hover:border-accent/40 shadow-sm hover:shadow-md hover:-translate-y-px cursor-default"
                    : "bg-surface border-surface-3/20 opacity-70 grayscale-[30%] hover:opacity-100"
                }
              `}>
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-5">
                      {/* Toggle */}
                      <button
                        onClick={() => handleToggle(d.config.domain)}
                        className={`w-12 h-6 rounded-full relative cursor-pointer transition-all duration-300 shadow-inner ${
                          d.config.enabled ? "bg-accent" : "bg-surface-3"
                        }`}>
                        <span
                          className={`absolute top-1 w-4 h-4 rounded-full bg-white shadow-sm transition-all duration-300 ${
                            d.config.enabled ? "left-7" : "left-1"
                          }`}
                        />
                      </button>

                      <div>
                        <div className="flex items-center gap-3">
                          <a
                            href={`https://${d.config.domain}`}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-text font-bold text-lg hover:text-accent transition-colors flex items-center group/link font-sans">
                            {d.config.domain}
                            <svg
                              className="w-3.5 h-3.5 ml-1.5 opacity-0 -translate-x-2 group-hover/link:opacity-100 group-hover/link:translate-x-0 transition-all duration-200"
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24">
                              <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2.5}
                                d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"
                              />
                            </svg>
                          </a>
                          {/* Cert Badge */}
                          <span
                            className={`flex items-center gap-1 text-[10px] uppercase font-bold tracking-wider px-2.5 py-1 rounded-md ${
                              d.cert_valid
                                ? "bg-success/15 text-success border border-success/20"
                                : "bg-danger/15 text-danger border border-danger/20"
                            }`}>
                            {d.cert_valid ? (
                              <>
                                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
                                </svg>{" "}Valid SSL
                              </>
                            ) : (
                              <>
                                <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M6 18L18 6M6 6l12 12" />
                                </svg>{" "}Invalid SSL
                              </>
                            )}
                          </span>
                          {/* CORS Badge */}
                          <button
                            onClick={() => handleToggleCors(d.config.domain)}
                            title={d.config.cors_enabled ? "CORS enabled — click to disable" : "Click to enable CORS headers"}
                            className={`flex items-center gap-1 text-[10px] uppercase font-bold tracking-wider px-2.5 py-1 rounded-md cursor-pointer transition-all ${
                              d.config.cors_enabled
                                ? "bg-accent/15 text-accent border border-accent/30 hover:bg-accent/25"
                                : "bg-surface-3/30 text-text-muted/50 border border-surface-3/20 hover:text-text-muted hover:bg-surface-3/50"
                            }`}>
                            CORS
                          </button>
                          {/* Stats */}
                          {stats[d.config.domain] && (
                            <span className="text-[10px] font-mono text-text-muted/60 px-2 py-1 bg-surface rounded-md border border-surface-3/20">
                              {stats[d.config.domain].count} req
                              {" · "}
                              {Math.round(stats[d.config.domain].totalMs / stats[d.config.domain].count)}ms
                            </span>
                          )}
                        </div>
                        <div className="inline-flex items-center text-sm text-text-muted mt-2 font-mono bg-surface px-2.5 py-1.5 rounded-md border border-surface-3/30">
                          <svg
                            className="w-3.5 h-3.5 mr-2 text-accent/70"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24">
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M13 5l7 7-7 7M5 5l7 7-7 7"
                            />
                          </svg>
                          {d.config.upstream}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-2 opacity-100 transition-opacity duration-200">
                      {tunnels[d.config.domain] ? (
                        <div className="flex flex-col gap-1.5 mr-2">
                          {tunnels[d.config.domain].loading ? (
                            <span className="text-xs text-accent animate-pulse">
                              Starting Tunnel...
                            </span>
                          ) : (
                            <>
                              <div className="flex items-center bg-accent/10 border border-accent/20 rounded-lg overflow-hidden">
                                <a
                                  href={tunnels[d.config.domain].url}
                                  target="_blank"
                                  className="px-3 py-1.5 text-xs font-mono text-accent hover:underline">
                                  {tunnels[d.config.domain].url}
                                </a>
                                <button
                                  onClick={() =>
                                    navigator.clipboard.writeText(
                                      tunnels[d.config.domain].url,
                                    )
                                  }
                                  className="px-2 border-l border-accent/20 text-accent hover:bg-accent/20 transition-colors"
                                  title="Copy URL">
                                  <svg
                                    className="w-3.5 h-3.5"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24">
                                    <path
                                      strokeLinecap="round"
                                      strokeLinejoin="round"
                                      strokeWidth={2}
                                      d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                                    />
                                  </svg>
                                </button>
                                <button
                                  onClick={() => toggleTunnel(d.config.domain)}
                                  className="px-2 border-l border-accent/20 text-accent hover:bg-accent hover:text-white transition-colors"
                                  title="Stop Tunnel">
                                  <svg
                                    className="w-4 h-4"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24">
                                    <path
                                      strokeLinecap="round"
                                      strokeLinejoin="round"
                                      strokeWidth={2}
                                      d="M6 18L18 6M6 6l12 12"
                                    />
                                  </svg>
                                </button>
                              </div>
                              <span className="text-[10px] text-text-muted/60 italic">
                                ⚠ Nếu không mở được, hãy đổi DNS sang 1.1.1.1
                                hoặc bật Secure DNS
                              </span>
                            </>
                          )}
                        </div>
                      ) : (
                        <button
                          onClick={() => toggleTunnel(d.config.domain)}
                          className="p-2.5 rounded-lg text-text-muted cursor-pointer hover:text-accent hover:bg-accent/10 border border-transparent hover:border-accent/20 transition-all"
                          title="Share Public Tunnel">
                          <svg
                            className="w-5 h-5"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24">
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                            />
                          </svg>
                        </button>
                      )}
                      <button
                        onClick={() => handleEdit(d)}
                        className="flex items-center gap-1.5 px-3 py-2 rounded-lg text-xs font-semibold text-accent bg-accent/10 border border-accent/20 hover:bg-accent hover:text-white cursor-pointer transition-all shadow-sm"
                        title="Edit Configuration">
                        <svg
                          className="w-4 h-4"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                          />
                        </svg>
                        Sửa
                      </button>
                      <button
                        onClick={() =>
                          navigator.clipboard.writeText(
                            `https://${d.config.domain}`,
                          )
                        }
                        className="p-2.5 rounded-lg text-text-muted cursor-pointer hover:text-accent hover:bg-accent/10 border border-transparent hover:border-accent/20 transition-all"
                        title="Copy URL">
                        <svg
                          className="w-5 h-5"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                          />
                        </svg>
                      </button>
                      <button
                        onClick={() => handleRemove(d.config.domain)}
                        className="p-2.5 rounded-lg text-text-muted cursor-pointer hover:text-danger hover:bg-danger/10 border border-transparent hover:border-danger/20 transition-all"
                        title="Remove Route">
                        <svg
                          className="w-5 h-5"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24">
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                          />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              ))}
            </div>

            {/* Log Viewer */}
            {showLogs && (
              <div className="mt-8 p-5 rounded-xl bg-[#0a0f1d] border border-surface-3 shadow-inner">
                <div className="flex items-center justify-between mb-3">
                  <h3 className="text-sm font-semibold text-text flex items-center gap-2">
                    <svg
                      className="w-4 h-4 text-accent"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24">
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        strokeWidth={2}
                        d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                      />
                    </svg>
                    nginx error.log
                  </h3>
                  <button
                    onClick={handleShowLogs}
                    className="text-xs text-text-muted/60 hover:text-text-muted transition-colors cursor-pointer font-mono">
                    ↻ Refresh
                  </button>
                </div>
                <div className="bg-black/50 rounded-lg p-4 h-64 overflow-y-auto font-mono text-[11px] text-gray-300 leading-normal border border-white/5">
                  {logs.length === 0 ? (
                    <p className="text-center py-10 text-text-muted/40 italic">
                      Nothing interesting is happening right now.
                    </p>
                  ) : (
                    <div className="space-y-1">
                      {logs.map((line, i) => (
                        <div
                          key={i}
                          className={`py-0.5 px-2 rounded ${line.toLowerCase().includes("error") ? "text-danger bg-danger/10" : "hover:bg-white/5"}`}>
                          {line}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}
          </>
        )}
      </div>

      {/* Delete Confirmation Modal */}
      {deleteConfirm && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm animate-in fade-in">
          <div className="bg-surface-2 border border-surface-3 rounded-2xl shadow-2xl p-6 w-full max-w-sm mx-4">
            <div className="flex items-center gap-3 mb-4">
              <span className="flex items-center justify-center w-10 h-10 rounded-xl bg-danger/15 text-danger">
                <svg
                  className="w-5 h-5"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                  />
                </svg>
              </span>
              <h3 className="text-lg font-bold text-text">Xác nhận xóa</h3>
            </div>
            <p className="text-sm text-text-muted mb-1">
              Bạn có chắc muốn xóa domain này?
            </p>
            <p className="text-sm font-mono font-semibold text-text bg-surface px-3 py-2 rounded-lg border border-surface-3/50 mb-6">
              {deleteConfirm}
            </p>
            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setDeleteConfirm(null)}
                className="px-5 py-2.5 rounded-lg text-sm font-semibold text-text-muted bg-surface-3/50 hover:bg-surface-3 transition-colors cursor-pointer">
                Hủy
              </button>
              <button
                onClick={confirmRemove}
                className="px-5 py-2.5 rounded-lg text-sm font-bold text-white bg-danger hover:bg-red-600 transition-colors cursor-pointer shadow-md shadow-danger/20 active:scale-95">
                Xóa domain
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
