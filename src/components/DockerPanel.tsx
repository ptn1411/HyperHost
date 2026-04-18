import { useEffect, useMemo, useState } from "react";
import { api, ComposeFileEntry, ComposeStatus, DockerStatus } from "../lib/tauri";

interface Props {
  projectPath: string;
  domain: string;
  onClose: () => void;
}

type DbKey = "redis" | "postgres" | "mysql" | "mongo" | "rabbitmq" | "elastic";

interface DbChoice {
  enabled: boolean;
  port: string;
  password: string;
  version: string;
}

const DB_DEFAULTS: Record<DbKey, { label: string; defaultPort: string; defaultVersion: string; needsPassword: boolean; slug: string }> = {
  redis:    { label: "Redis",         defaultPort: "6379",  defaultVersion: "7-alpine",     needsPassword: true,  slug: "redis" },
  postgres: { label: "PostgreSQL",    defaultPort: "5432",  defaultVersion: "16-alpine",    needsPassword: true,  slug: "postgres" },
  mysql:    { label: "MySQL",         defaultPort: "3306",  defaultVersion: "8",            needsPassword: true,  slug: "mysql" },
  mongo:    { label: "MongoDB",       defaultPort: "27017", defaultVersion: "7",            needsPassword: true,  slug: "mongo" },
  rabbitmq: { label: "RabbitMQ",      defaultPort: "5672",  defaultVersion: "3-management", needsPassword: true,  slug: "rabbitmq" },
  elastic:  { label: "Elasticsearch", defaultPort: "9200",  defaultVersion: "8.13.0",       needsPassword: true,  slug: "elastic" },
};

const initialDbState = (): Record<DbKey, DbChoice> => {
  const out = {} as Record<DbKey, DbChoice>;
  (Object.keys(DB_DEFAULTS) as DbKey[]).forEach((k) => {
    out[k] = {
      enabled: false,
      port: DB_DEFAULTS[k].defaultPort,
      password: DB_DEFAULTS[k].needsPassword ? "devpass" : "",
      version: DB_DEFAULTS[k].defaultVersion,
    };
  });
  return out;
};

type BusyKey = `${"up" | "down" | "restart" | "logs"}:${string}` | "refresh" | "save";

export function DockerPanel({ projectPath, domain, onClose }: Props) {
  const [docker, setDocker] = useState<DockerStatus | null>(null);
  const [status, setStatus] = useState<ComposeStatus | null>(null);
  const [busy, setBusy] = useState<BusyKey | null>(null);
  const [output, setOutput] = useState<string>("");
  const [error, setError] = useState<string | null>(null);

  const [promptOpen, setPromptOpen] = useState(false);
  const [dbs, setDbs] = useState<Record<DbKey, DbChoice>>(initialDbState);
  const [extraReq, setExtraReq] = useState<string>("");
  const [copied, setCopied] = useState(false);

  const [pasteName, setPasteName] = useState<string>("docker-compose.yml");
  const [pasteContent, setPasteContent] = useState<string>("");
  const [saveMsg, setSaveMsg] = useState<string | null>(null);
  const [saveErr, setSaveErr] = useState<string | null>(null);

  const refresh = async () => {
    setBusy("refresh");
    setError(null);
    try {
      const [d, s] = await Promise.all([
        api.dockerCheck(),
        api.composeStatus(projectPath),
      ]);
      setDocker(d);
      setStatus(s);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setBusy(null);
    }
  };

  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectPath]);

  const runFileAction = async (
    kind: "up" | "down" | "restart" | "logs",
    file: string,
    fn: () => Promise<string>,
  ) => {
    setBusy(`${kind}:${file}`);
    setError(null);
    setOutput("");
    try {
      const out = await fn();
      setOutput(`[${file}]\n${out}`);
      if (kind !== "logs") {
        const s = await api.composeStatus(projectPath);
        setStatus(s);
      }
    } catch (e: any) {
      setError(`[${file}] ${String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  const generatedPrompt = useMemo(
    () => buildPrompt(domain, projectPath, dbs, extraReq),
    [domain, projectPath, dbs, extraReq],
  );

  const suggestedFileName = useMemo(() => {
    const sel = (Object.keys(dbs) as DbKey[]).filter((k) => dbs[k].enabled);
    if (sel.length === 0) return "docker-compose.yml";
    if (sel.length === 1) return `docker-compose.${DB_DEFAULTS[sel[0]].slug}.yml`;
    return "docker-compose.yml";
  }, [dbs]);

  useEffect(() => {
    if (promptOpen) {
      setPasteName(suggestedFileName);
    }
  }, [promptOpen, suggestedFileName]);

  const copyPrompt = async () => {
    try {
      await navigator.clipboard.writeText(generatedPrompt);
      setCopied(true);
      setTimeout(() => setCopied(false), 1800);
    } catch {
      // fallback
    }
  };

  const handleSavePaste = async () => {
    setSaveErr(null);
    setSaveMsg(null);
    if (!pasteName.trim() || !pasteContent.trim()) {
      setSaveErr("Cần điền tên file và nội dung YAML.");
      return;
    }
    const exists = status?.files.some((f) => f.name.toLowerCase() === pasteName.trim().toLowerCase());
    if (exists && !confirm(`File ${pasteName} đã tồn tại. Ghi đè?`)) {
      return;
    }
    setBusy("save");
    try {
      const path = await api.composeSaveFile(projectPath, pasteName.trim(), pasteContent);
      setSaveMsg(`Đã lưu: ${path}`);
      setPasteContent("");
      const s = await api.composeStatus(projectPath);
      setStatus(s);
    } catch (e: any) {
      setSaveErr(String(e));
    } finally {
      setBusy(null);
    }
  };

  const anySelected = Object.values(dbs).some((d) => d.enabled);
  const noFiles = !status || status.files.length === 0;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={onClose}>
      <div className="bg-surface-2 border border-surface-3 rounded-2xl shadow-2xl w-full max-w-3xl mx-4 max-h-[90vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
        {/* Header */}
        <div className="p-5 border-b border-surface-3/40 flex items-start justify-between gap-3">
          <div className="min-w-0">
            <h3 className="text-lg font-bold text-text flex items-center gap-2">
              <svg className="w-5 h-5 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 8h14M5 12h14M5 16h14M3 4h18a1 1 0 011 1v14a1 1 0 01-1 1H3a1 1 0 01-1-1V5a1 1 0 011-1z" />
              </svg>
              Docker · {domain}
            </h3>
            <p className="text-[11px] text-text-muted font-mono truncate mt-1">{projectPath}</p>
          </div>
          <button onClick={onClose} className="text-text-muted hover:text-text px-2 py-1 cursor-pointer">✕</button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-5 space-y-4">
          {/* Docker daemon status */}
          <div className="flex items-center gap-3 text-xs">
            <span className="text-text-muted">Docker:</span>
            {docker === null ? (
              <span className="text-text-muted italic">Đang kiểm tra…</span>
            ) : !docker.installed ? (
              <span className="text-danger font-semibold">Chưa cài đặt</span>
            ) : !docker.daemon_running ? (
              <span className="text-warning font-semibold">CLI có nhưng daemon không chạy</span>
            ) : (
              <span className="text-success font-semibold">Sẵn sàng · {docker.version ?? ""}</span>
            )}
            <button
              onClick={refresh}
              disabled={busy !== null}
              className="ml-auto px-3 py-1 rounded-md text-[11px] font-semibold text-text-muted hover:text-text hover:bg-surface-3 cursor-pointer disabled:opacity-40">
              {busy === "refresh" ? "Đang refresh…" : "Refresh"}
            </button>
            <button
              onClick={() => setPromptOpen(true)}
              className="px-3 py-1 rounded-md text-[11px] font-bold text-accent border border-accent/30 hover:bg-accent/10 cursor-pointer">
              Generate AI prompt
            </button>
          </div>

          {/* Compose files list */}
          {noFiles ? (
            <div className="rounded-lg bg-surface border border-surface-3/40 p-4 text-center">
              <p className="text-xs text-text-muted">Chưa có file compose trong project.</p>
              <button
                onClick={() => setPromptOpen(true)}
                className="mt-2 text-[11px] font-semibold text-accent hover:underline cursor-pointer">
                Tạo prompt cho AI →
              </button>
            </div>
          ) : (
            <div className="space-y-3">
              {status!.files.map((f) => (
                <ComposeFileCard
                  key={f.path}
                  file={f}
                  busy={busy}
                  daemonRunning={!!docker?.daemon_running}
                  onUp={() => runFileAction("up", f.name, () => api.composeUp(projectPath, f.name))}
                  onDown={() => runFileAction("down", f.name, () => api.composeDown(projectPath, f.name))}
                  onRestart={() => runFileAction("restart", f.name, () => api.composeRestart(projectPath, f.name))}
                  onLogs={() => runFileAction("logs", f.name, () => api.composeLogs(projectPath, f.name, 200))}
                />
              ))}
            </div>
          )}

          {error && (
            <pre className="text-xs text-danger bg-danger/10 border border-danger/20 rounded-lg p-3 font-mono whitespace-pre-wrap break-words max-h-48 overflow-y-auto">
              {error}
            </pre>
          )}
          {output && (
            <pre className="text-[11px] text-text-muted bg-surface border border-surface-3/40 rounded-lg p-3 font-mono whitespace-pre-wrap break-words max-h-72 overflow-y-auto">
              {output}
            </pre>
          )}
        </div>
      </div>

      {/* Nested AI-prompt modal */}
      {promptOpen && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/70 backdrop-blur-sm" onClick={() => setPromptOpen(false)}>
          <div className="bg-surface-2 border border-surface-3 rounded-2xl shadow-2xl w-full max-w-2xl mx-4 max-h-[90vh] flex flex-col" onClick={(e) => e.stopPropagation()}>
            <div className="p-5 border-b border-surface-3/40 flex items-center justify-between">
              <h3 className="text-lg font-bold text-text">Tạo prompt AI cho docker-compose.yml</h3>
              <button onClick={() => setPromptOpen(false)} className="text-text-muted hover:text-text px-2 py-1 cursor-pointer">✕</button>
            </div>

            <div className="flex-1 overflow-y-auto p-5 space-y-4">
              <p className="text-xs text-text-muted">
                Chọn DB → Copy prompt → dán vào ChatGPT/Claude → AI trả YAML → dán vào ô bên dưới và bấm Lưu.
              </p>

              <div className="space-y-2">
                {(Object.keys(DB_DEFAULTS) as DbKey[]).map((k) => {
                  const d = dbs[k];
                  const meta = DB_DEFAULTS[k];
                  return (
                    <div key={k} className="rounded-lg border border-surface-3/40 bg-surface p-3">
                      <label className="flex items-center gap-2 cursor-pointer">
                        <input
                          type="checkbox"
                          checked={d.enabled}
                          onChange={(e) =>
                            setDbs((prev) => ({ ...prev, [k]: { ...prev[k], enabled: e.target.checked } }))
                          }
                          className="accent-accent"
                        />
                        <span className="text-sm font-semibold text-text">{meta.label}</span>
                      </label>
                      {d.enabled && (
                        <div className="grid grid-cols-3 gap-2 mt-2 pl-6">
                          <Input label="Version" value={d.version}
                            onChange={(v) => setDbs((p) => ({ ...p, [k]: { ...p[k], version: v } }))} />
                          <Input label="Host port" value={d.port}
                            onChange={(v) => setDbs((p) => ({ ...p, [k]: { ...p[k], port: v } }))} />
                          {meta.needsPassword && (
                            <Input label="Password" value={d.password}
                              onChange={(v) => setDbs((p) => ({ ...p, [k]: { ...p[k], password: v } }))} />
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              <div>
                <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-1.5 uppercase">
                  Yêu cầu thêm (tùy chọn)
                </label>
                <textarea
                  value={extraReq}
                  onChange={(e) => setExtraReq(e.target.value)}
                  rows={3}
                  placeholder="Ví dụ: tên network là myapp_net, mount volume vào ./data, expose Redis Insight kèm theo…"
                  className="w-full px-3 py-2 rounded-lg bg-surface border border-surface-3 text-text font-mono text-xs focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 resize-y"
                />
              </div>

              <div>
                <div className="flex items-center justify-between mb-1.5">
                  <span className="text-[11px] font-bold tracking-wider text-text-muted uppercase">Prompt</span>
                  <button
                    onClick={copyPrompt}
                    disabled={!anySelected}
                    className={`px-3 py-1 rounded text-[11px] font-bold cursor-pointer disabled:opacity-40 transition-all ${
                      copied ? "bg-success text-white" : "bg-accent text-white hover:bg-accent-hover"
                    }`}>
                    {copied ? "Đã copy ✓" : "Copy prompt"}
                  </button>
                </div>
                <pre className="text-[11px] text-text bg-surface border border-surface-3/40 rounded-lg p-3 font-mono whitespace-pre-wrap break-words max-h-64 overflow-y-auto">
                  {generatedPrompt}
                </pre>
              </div>

              {/* Paste & save */}
              <div className="border-t border-surface-3/40 pt-4 space-y-2">
                <div className="flex items-center justify-between">
                  <span className="text-[11px] font-bold tracking-wider text-text-muted uppercase">
                    Dán YAML từ AI và lưu
                  </span>
                  <span className="text-[10px] text-text-muted">
                    Lưu vào: <code className="font-mono bg-surface-3/40 px-1 rounded">{projectPath}</code>
                  </span>
                </div>

                <div className="flex gap-2">
                  <input
                    type="text"
                    value={pasteName}
                    onChange={(e) => setPasteName(e.target.value)}
                    placeholder="docker-compose.yml"
                    className="flex-1 px-3 py-1.5 rounded-md bg-surface border border-surface-3 text-text font-mono text-xs focus:outline-none focus:border-accent"
                  />
                  <button
                    onClick={handleSavePaste}
                    disabled={busy === "save" || !pasteContent.trim() || !pasteName.trim()}
                    className="px-4 py-1.5 rounded-md text-xs font-bold text-white bg-success/80 hover:bg-success disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-all">
                    {busy === "save" ? "Đang lưu…" : "Lưu YAML"}
                  </button>
                </div>

                <textarea
                  value={pasteContent}
                  onChange={(e) => setPasteContent(e.target.value)}
                  rows={8}
                  placeholder="# Dán nội dung docker-compose.yml ở đây…"
                  className="w-full px-3 py-2 rounded-lg bg-surface border border-surface-3 text-text font-mono text-xs focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 resize-y"
                />

                {saveErr && (
                  <p className="text-[11px] text-danger bg-danger/10 border border-danger/20 rounded px-2 py-1 font-mono break-words">
                    {saveErr}
                  </p>
                )}
                {saveMsg && (
                  <p className="text-[11px] text-success bg-success/10 border border-success/20 rounded px-2 py-1 font-mono break-words">
                    {saveMsg}
                  </p>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function ComposeFileCard({
  file,
  busy,
  daemonRunning,
  onUp,
  onDown,
  onRestart,
  onLogs,
}: {
  file: ComposeFileEntry;
  busy: BusyKey | null;
  daemonRunning: boolean;
  onUp: () => void;
  onDown: () => void;
  onRestart: () => void;
  onLogs: () => void;
}) {
  const disabled = !daemonRunning || busy !== null;
  const isBusy = (kind: "up" | "down" | "restart" | "logs") => busy === `${kind}:${file.name}`;
  return (
    <div className="rounded-lg bg-surface border border-surface-3/40 p-3">
      <div className="flex items-center justify-between gap-2 mb-2">
        <div className="min-w-0">
          <div className="text-xs font-semibold text-text">{file.name}</div>
          <div className="text-[10px] text-text-muted font-mono truncate">{file.path}</div>
        </div>
        <div className="flex flex-wrap gap-1.5 shrink-0">
          <button
            onClick={onUp}
            disabled={disabled}
            className="px-2.5 py-1 rounded text-[11px] font-bold text-white bg-success/80 hover:bg-success disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-all">
            {isBusy("up") ? "…" : "Up -d"}
          </button>
          <button
            onClick={onRestart}
            disabled={disabled}
            className="px-2.5 py-1 rounded text-[11px] font-bold text-text bg-surface-3 hover:bg-surface-3/70 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-all">
            {isBusy("restart") ? "…" : "Restart"}
          </button>
          <button
            onClick={onDown}
            disabled={disabled}
            className="px-2.5 py-1 rounded text-[11px] font-bold text-white bg-danger/80 hover:bg-danger disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-all">
            {isBusy("down") ? "…" : "Down"}
          </button>
          <button
            onClick={onLogs}
            disabled={disabled}
            className="px-2.5 py-1 rounded text-[11px] font-bold text-text bg-surface-3 hover:bg-surface-3/70 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer transition-all">
            {isBusy("logs") ? "…" : "Logs"}
          </button>
        </div>
      </div>

      {file.services.length > 0 ? (
        <div className="border-t border-surface-3/30 pt-2 space-y-1">
          {file.services.map((s) => (
            <div key={s.name} className="flex items-center gap-2 text-xs">
              <span className={`w-2 h-2 rounded-full shrink-0 ${
                s.state === "running" ? "bg-success" :
                s.state === "exited" ? "bg-danger" : "bg-text-muted"
              }`} />
              <span className="font-semibold text-text">{s.name}</span>
              <span className="text-text-muted truncate">{s.image}</span>
              {s.ports && (
                <span className="ml-auto font-mono text-[10px] text-accent">{s.ports}</span>
              )}
            </div>
          ))}
        </div>
      ) : (
        <p className="text-[11px] text-text-muted italic border-t border-surface-3/30 pt-2">
          Chưa có service nào đang chạy. Bấm Up -d để khởi động.
        </p>
      )}
    </div>
  );
}

function Input({ label, value, onChange }: { label: string; value: string; onChange: (v: string) => void }) {
  return (
    <label className="block">
      <span className="text-[10px] uppercase tracking-wider text-text-muted">{label}</span>
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full px-2 py-1 mt-0.5 rounded bg-surface-3/40 border border-surface-3 text-text font-mono text-xs focus:outline-none focus:border-accent"
      />
    </label>
  );
}

function buildPrompt(
  domain: string,
  projectPath: string,
  dbs: Record<DbKey, DbChoice>,
  extra: string,
): string {
  const selected = (Object.keys(dbs) as DbKey[]).filter((k) => dbs[k].enabled);
  if (selected.length === 0) {
    return "Chọn ít nhất một DB ở trên để xem prompt.";
  }

  const lines: string[] = [];
  lines.push(`Tôi cần file \`docker-compose.yml\` cho dự án local "${domain}".`);
  lines.push(`Thư mục dự án: ${projectPath}`);
  lines.push("");
  lines.push("Các service cần có:");
  for (const k of selected) {
    const d = dbs[k];
    const meta = DB_DEFAULTS[k];
    const parts = [
      `${meta.label} ${d.version}`,
      `bind host port ${d.port}`,
    ];
    if (meta.needsPassword && d.password) {
      parts.push(`password "${d.password}"`);
    }
    parts.push(`volume riêng để persist data`);
    lines.push(`- ${parts.join(", ")}`);
  }
  lines.push("");
  lines.push("Yêu cầu chung:");
  lines.push("- Mỗi service có healthcheck phù hợp.");
  lines.push("- restart: unless-stopped.");
  lines.push("- Đặt tên service ngắn gọn (vd `redis`, `postgres`).");
  lines.push("- Bind các port ra 127.0.0.1 (chỉ truy cập từ máy host, không expose mạng ngoài).");
  lines.push("- Dùng named volumes (không bind mount thư mục local) để dễ xóa.");
  lines.push("- Kèm comment ngắn ở đầu file nói rõ cách up/down.");
  if (extra.trim()) {
    lines.push(`- ${extra.trim()}`);
  }
  lines.push("");
  lines.push("Trả lời CHỈ nội dung file `docker-compose.yml` trong code block, không kèm lời giải thích.");

  return lines.join("\n");
}
