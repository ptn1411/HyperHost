import { useEffect, useMemo, useState } from "react";
import { api, PortInfo, ProjectInfo, Template } from "../lib/tauri";

export interface QuickStartSelection {
  domain: string;
  upstream: string;
  advancedConfig: string;
  openEditor: boolean;
  projectPath?: string;
  runCommand?: string;
}

interface Props {
  onPick: (sel: QuickStartSelection) => void;
}

type TabKey = "templates" | "ports" | "projects";

export function QuickStartPanel({ onPick }: Props) {
  const [open, setOpen] = useState(false);
  const [tab, setTab] = useState<TabKey>("templates");

  const [templates, setTemplates] = useState<Template[]>([]);
  const [ports, setPorts] = useState<PortInfo[] | null>(null);
  const [scanningPorts, setScanningPorts] = useState(false);
  const [hideSystemPorts, setHideSystemPorts] = useState(true);

  const [scanPath, setScanPath] = useState<string>("");
  const [projects, setProjects] = useState<ProjectInfo[] | null>(null);
  const [scanningProjects, setScanningProjects] = useState(false);
  const [scanError, setScanError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    if (templates.length === 0) {
      api.listTemplates().then(setTemplates).catch(() => {});
    }
    if (!scanPath) {
      api.getHomeDir().then((h) => setScanPath(h)).catch(() => {});
    }
  }, [open]);

  const handleScanPorts = async () => {
    setScanningPorts(true);
    try {
      const p = await api.scanPorts();
      setPorts(p);
    } catch (e) {
      console.error(e);
    } finally {
      setScanningPorts(false);
    }
  };

  const handleScanProjects = async () => {
    if (!scanPath.trim()) return;
    setScanningProjects(true);
    setScanError(null);
    try {
      const p = await api.scanProjects(scanPath.trim(), 3);
      setProjects(p);
    } catch (e: any) {
      setScanError(String(e));
      setProjects(null);
    } finally {
      setScanningProjects(false);
    }
  };

  const pickTemplate = (t: Template) => {
    onPick({
      domain: "",
      upstream: t.default_upstream,
      advancedConfig: t.advanced_config ?? "",
      openEditor: !!t.advanced_config,
    });
  };

  const pickPort = (p: PortInfo) => {
    onPick({
      domain: "",
      upstream: `http://127.0.0.1:${p.port}`,
      advancedConfig: "",
      openEditor: false,
    });
  };

  const pickProject = (p: ProjectInfo) => {
    onPick({
      domain: p.suggested_domain,
      upstream: `http://127.0.0.1:${p.suggested_port}`,
      advancedConfig: "",
      openEditor: false,
      projectPath: p.path,
      runCommand: p.suggested_command ?? undefined,
    });
  };

  const visiblePorts = useMemo(() => {
    if (!ports) return [];
    if (!hideSystemPorts) return ports;
    const NOISE = new Set([
      "svchost", "system", "system idle process", "lsass", "services",
      "spoolsv", "wininit", "smss", "csrss", "winlogon", "explorer",
      "nginx", "hyperhost", "hyperhost-app",
      "rundll32", "dllhost", "fontdrvhost", "searchhost",
      "msmpeng", "vmcompute", "vmms",
    ]);
    return ports.filter((p) => {
      if (p.port < 1024) return false;
      const name = (p.process ?? "").toLowerCase();
      if (!name) return true;
      return !NOISE.has(name);
    });
  }, [ports, hideSystemPorts]);

  const templateCategories = useMemo(() => {
    const groups: Record<string, Template[]> = {};
    for (const t of templates) {
      (groups[t.category] ||= []).push(t);
    }
    return Object.entries(groups);
  }, [templates]);

  return (
    <div className="mb-6 rounded-xl bg-surface-2 border border-surface-3/30 overflow-hidden">
      <button
        onClick={() => setOpen((v) => !v)}
        className="w-full flex items-center justify-between px-5 py-3.5 hover:bg-surface-3/30 transition-colors cursor-pointer">
        <div className="flex items-center gap-2.5">
          <svg className="w-4 h-4 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
          <span className="text-sm font-semibold text-text">Quick Start</span>
          <span className="text-xs text-text-muted">
            Templates · Port scan · Project scan
          </span>
        </div>
        <svg
          className={`w-4 h-4 text-text-muted transition-transform ${open ? "rotate-180" : ""}`}
          fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {open && (
        <div className="border-t border-surface-3/40">
          {/* Tabs */}
          <div className="flex gap-1 p-1 bg-surface-3/20 border-b border-surface-3/30">
            {(["templates", "ports", "projects"] as TabKey[]).map((k) => (
              <button
                key={k}
                onClick={() => setTab(k)}
                className={`px-4 py-1.5 rounded-md text-xs font-semibold transition-all ${
                  tab === k
                    ? "bg-surface text-text shadow-sm"
                    : "text-text-muted hover:text-text cursor-pointer"
                }`}>
                {k === "templates" && "Templates"}
                {k === "ports" && "Ports đang mở"}
                {k === "projects" && "Quét dự án"}
              </button>
            ))}
          </div>

          <div className="p-5">
            {tab === "templates" && (
              <div className="space-y-5">
                <p className="text-xs text-text-muted">
                  Chọn preset để tự điền upstream. Các stack có HMR/WebSocket sẽ mở sẵn editor với nginx snippet phù hợp.
                </p>
                {templates.length === 0 && (
                  <p className="text-xs text-text-muted italic">Loading…</p>
                )}
                {templateCategories.map(([cat, items]) => (
                  <div key={cat}>
                    <h4 className="text-[11px] uppercase tracking-wider font-bold text-text-muted mb-2">{cat}</h4>
                    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                      {items.map((t) => (
                        <button
                          key={t.id}
                          onClick={() => pickTemplate(t)}
                          className="text-left p-3 rounded-lg bg-surface border border-surface-3/40 hover:border-accent/40 hover:bg-accent/5 transition-all cursor-pointer group">
                          <div className="flex items-center justify-between mb-1">
                            <span className="text-sm font-semibold text-text group-hover:text-accent transition-colors">
                              {t.name}
                            </span>
                            {t.advanced_config && (
                              <span className="text-[9px] uppercase tracking-wider font-bold px-1.5 py-0.5 rounded bg-accent/10 text-accent border border-accent/20">
                                WS
                              </span>
                            )}
                          </div>
                          <p className="text-[11px] text-text-muted font-mono truncate">
                            {t.default_upstream.replace("http://127.0.0.1:", ":")}
                          </p>
                          <p className="text-[11px] text-text-muted/70 mt-1 line-clamp-2">{t.description}</p>
                        </button>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {tab === "ports" && (
              <div className="space-y-3">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-xs text-text-muted">
                    Liệt kê tất cả TCP port đang lắng nghe trên 127.0.0.1 cùng tên process.
                  </p>
                  <div className="shrink-0 flex items-center gap-2">
                    <label className="flex items-center gap-1.5 text-[11px] text-text-muted cursor-pointer">
                      <input
                        type="checkbox"
                        checked={hideSystemPorts}
                        onChange={(e) => setHideSystemPorts(e.target.checked)}
                        className="accent-accent"
                      />
                      Ẩn system / nginx
                    </label>
                    <button
                      onClick={handleScanPorts}
                      disabled={scanningPorts}
                      className="px-4 py-1.5 rounded-lg text-xs font-semibold bg-accent text-white hover:bg-accent-hover disabled:opacity-50 cursor-pointer transition-all">
                      {scanningPorts ? "Đang quét…" : ports ? "Quét lại" : "Quét ngay"}
                    </button>
                  </div>
                </div>

                {ports !== null && visiblePorts.length === 0 && (
                  <p className="text-sm text-text-muted italic py-4 text-center bg-surface rounded-lg border border-dashed border-surface-3/40">
                    {ports.length === 0
                      ? "Không có port nào đang lắng nghe."
                      : "Tất cả port đều bị ẩn — bỏ chọn lọc để xem."}
                  </p>
                )}

                {visiblePorts.length > 0 && (
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-2">
                    {visiblePorts.map((p) => (
                      <button
                        key={p.port}
                        onClick={() => pickPort(p)}
                        className="group flex items-center justify-between p-3 rounded-lg bg-surface border border-surface-3/40 hover:border-accent/40 hover:bg-accent/5 transition-all cursor-pointer text-left">
                        <div className="flex items-center gap-3 min-w-0">
                          <span className="font-mono font-bold text-accent text-sm shrink-0">:{p.port}</span>
                          <div className="min-w-0">
                            {p.process && (
                              <div className="text-xs font-semibold text-text truncate">
                                {p.process}
                                {p.pid !== null && (
                                  <span className="ml-1.5 text-[10px] font-mono text-text-muted/70">pid {p.pid}</span>
                                )}
                              </div>
                            )}
                            {p.guess && (
                              <div className="text-[11px] text-text-muted truncate">{p.guess}</div>
                            )}
                          </div>
                        </div>
                        <span className="text-[10px] uppercase tracking-wider font-bold text-text-muted group-hover:text-accent shrink-0 ml-2">
                          Dùng →
                        </span>
                      </button>
                    ))}
                  </div>
                )}
              </div>
            )}

            {tab === "projects" && (
              <div className="space-y-3">
                <p className="text-xs text-text-muted">
                  Quét thư mục để tìm các dự án Node / Rust / Go / Django / Laravel / Rails… Nhận diện port mặc định theo framework.
                </p>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={scanPath}
                    onChange={(e) => setScanPath(e.target.value)}
                    onKeyDown={(e) => { if (e.key === "Enter") handleScanProjects(); }}
                    placeholder="C:\Users\you\Code"
                    className="flex-1 px-3 py-2 rounded-lg bg-surface border border-surface-3 text-text font-mono text-xs focus:outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 transition-all"
                  />
                  <button
                    onClick={handleScanProjects}
                    disabled={scanningProjects || !scanPath.trim()}
                    className="px-4 py-2 rounded-lg text-xs font-semibold bg-accent text-white hover:bg-accent-hover disabled:opacity-50 cursor-pointer transition-all">
                    {scanningProjects ? "Đang quét…" : "Quét"}
                  </button>
                </div>

                {scanError && (
                  <p className="text-xs text-danger bg-danger/10 border border-danger/20 rounded-lg p-2.5 font-mono">
                    {scanError}
                  </p>
                )}

                {projects !== null && projects.length === 0 && !scanError && (
                  <p className="text-sm text-text-muted italic py-4 text-center bg-surface rounded-lg border border-dashed border-surface-3/40">
                    Không tìm thấy dự án nào trong thư mục này.
                  </p>
                )}

                {projects && projects.length > 0 && (
                  <div className="max-h-80 overflow-y-auto space-y-1.5 pr-1">
                    {projects.map((p) => (
                      <ProjectRow key={p.path} project={p} onPick={pickProject} />
                    ))}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function ProjectRow({
  project,
  onPick,
}: {
  project: ProjectInfo;
  onPick: (p: ProjectInfo) => void;
}) {
  const [terminalError, setTerminalError] = useState<string | null>(null);
  const [busy, setBusy] = useState<"none" | "terminal" | "run">("none");

  const openTerminal = async (withCommand: boolean) => {
    setTerminalError(null);
    setBusy(withCommand ? "run" : "terminal");
    try {
      await api.openTerminal(
        project.path,
        withCommand ? project.suggested_command ?? undefined : undefined,
      );
    } catch (e: any) {
      setTerminalError(String(e));
    } finally {
      setBusy("none");
    }
  };

  return (
    <div className="group p-2.5 rounded-lg bg-surface border border-surface-3/40 hover:border-accent/40 hover:bg-accent/5 transition-all">
      <div className="flex items-center justify-between gap-3">
        <button
          onClick={() => onPick(project)}
          className="min-w-0 flex-1 text-left cursor-pointer"
          title="Tạo domain cho dự án này">
          <div className="flex items-center gap-2">
            <span className="text-sm font-semibold text-text truncate">{project.name}</span>
            <span className="text-[9px] uppercase tracking-wider font-bold px-1.5 py-0.5 rounded bg-accent/10 text-accent border border-accent/20 shrink-0">
              {project.kind}
            </span>
          </div>
          <p className="text-[11px] text-text-muted/70 font-mono truncate">{project.path}</p>
        </button>

        <div className="flex items-center gap-1 shrink-0">
          <div className="text-right mr-2">
            <div className="text-xs font-mono font-bold text-accent">:{project.suggested_port}</div>
            <div className="text-[10px] text-text-muted font-mono truncate max-w-[120px]">
              {project.suggested_domain}
            </div>
          </div>

          <button
            onClick={() => openTerminal(false)}
            disabled={busy !== "none"}
            title="Mở terminal tại thư mục dự án"
            className="p-1.5 rounded-md text-text-muted hover:text-accent hover:bg-accent/10 border border-transparent hover:border-accent/20 transition-all cursor-pointer disabled:opacity-50">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          </button>

          {project.suggested_command && (
            <button
              onClick={() => openTerminal(true)}
              disabled={busy !== "none"}
              title={`Mở terminal và chạy: ${project.suggested_command}`}
              className="flex items-center gap-1 px-2 py-1.5 rounded-md text-[11px] font-semibold text-white bg-success/80 hover:bg-success border border-success/30 transition-all cursor-pointer disabled:opacity-50">
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Run
            </button>
          )}

          <button
            onClick={() => onPick(project)}
            title="Tạo domain"
            className="px-2 py-1.5 rounded-md text-[11px] font-semibold text-accent bg-accent/10 border border-accent/20 hover:bg-accent hover:text-white transition-all cursor-pointer">
            +
          </button>
        </div>
      </div>

      {project.suggested_command && (
        <p className="mt-1.5 text-[10px] font-mono text-text-muted/60 truncate">
          <span className="text-text-muted">$</span> {project.suggested_command}
        </p>
      )}

      {terminalError && (
        <p className="mt-1.5 text-[11px] text-danger bg-danger/10 border border-danger/20 rounded px-2 py-1 font-mono">
          {terminalError}
        </p>
      )}
    </div>
  );
}
