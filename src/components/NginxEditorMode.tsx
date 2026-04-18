import { useState, useEffect } from "react";
import Editor from "@monaco-editor/react";
import { api } from "../lib/tauri";
import { i18n } from "../translation";

interface NginxEditorModeProps {
  initialDomain?: string;
  initialUpstream?: string;
  initialAdvancedConfig?: string;
  initialProjectPath?: string;
  initialRunCommand?: string;
  isEditing?: boolean;
  loading: boolean;
  onSave: (
    domain: string,
    upstream: string,
    advancedConfig: string,
    projectPath: string,
    runCommand: string,
  ) => Promise<void>;
  onCancel: () => void;
}

const FULL_TEMPLATE = `server {
    listen 443 ssl;
    http2  on;
    server_name $DOMAIN;

    ssl_certificate     "$CERT_PATH";
    ssl_certificate_key "$KEY_PATH";
    ssl_protocols       TLSv1.2 TLSv1.3;
    ssl_ciphers         HIGH:!aNULL:!MD5;
    ssl_session_cache   shared:SSL:1m;

    # WebSocket support
    proxy_http_version 1.1;
    proxy_set_header   Upgrade    $http_upgrade;
    proxy_set_header   Connection "upgrade";
    proxy_set_header   Host       $host;
    proxy_set_header   X-Real-IP  $remote_addr;
    proxy_read_timeout 300s;
    proxy_send_timeout 300s;

    location / {
        proxy_pass $UPSTREAM;
    }
}`;

export function NginxEditorMode({
  initialDomain = "",
  initialUpstream = "http://127.0.0.1:3000",
  initialAdvancedConfig = "",
  initialProjectPath = "",
  initialRunCommand = "",
  isEditing = false,
  loading,
  onSave,
  onCancel
}: NginxEditorModeProps) {
  const [domain, setDomain] = useState(initialDomain);
  const [upstream, setUpstream] = useState(initialUpstream);
  const [advancedConfig, setAdvancedConfig] = useState(initialAdvancedConfig);
  const [projectPath, setProjectPath] = useState(initialProjectPath);
  const [runCommand, setRunCommand] = useState(initialRunCommand);

  // Import modal state
  const [importOpen, setImportOpen] = useState(false);
  const [importText, setImportText] = useState("");
  const [importBusy, setImportBusy] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);

  // Validation state
  const [validateBusy, setValidateBusy] = useState(false);
  const [validateError, setValidateError] = useState<string | null>(null);
  const [validateOk, setValidateOk] = useState<string | null>(null);

  // Export modal state
  const [exportOpen, setExportOpen] = useState(false);
  const [exportProdDomain, setExportProdDomain] = useState("");
  const [exportProdUpstream, setExportProdUpstream] = useState("http://127.0.0.1:3000");
  const [exportBusy, setExportBusy] = useState(false);
  const [exportResult, setExportResult] = useState<string | null>(null);
  const [exportError, setExportError] = useState<string | null>(null);

  const handleValidate = async (content?: string): Promise<boolean> => {
    const src = (content ?? advancedConfig).trim();
    if (!src) {
      setValidateError("Config trống — không có gì để validate.");
      setValidateOk(null);
      return false;
    }
    setValidateBusy(true);
    setValidateError(null);
    setValidateOk(null);
    try {
      const out = await api.validateNginxConfig(src);
      setValidateOk(out.trim() || "nginx -t: syntax OK");
      return true;
    } catch (e: any) {
      setValidateError(String(e));
      return false;
    } finally {
      setValidateBusy(false);
    }
  };

  const handleImportApply = async () => {
    if (!importText.trim()) return;
    setImportBusy(true);
    setImportError(null);
    try {
      const r = await api.importNginxConfigText(importText);
      try {
        await api.validateNginxConfig(r.advanced_config);
        setValidateOk("Import passed nginx -t ✓");
        setValidateError(null);
      } catch (ve: any) {
        setValidateError(`Import: nginx -t thất bại\n${String(ve)}`);
        setValidateOk(null);
      }
      setAdvancedConfig(r.advanced_config);
      if (r.suggested_upstream) setUpstream(r.suggested_upstream);
      if (!domain.trim() && r.server_name) {
        const parts = r.server_name.split(".");
        const base = parts.length > 1 ? parts[0] : r.server_name;
        setDomain(`${base}.test`);
      }
      setImportOpen(false);
      setImportText("");
    } catch (e: any) {
      setImportError(String(e));
    } finally {
      setImportBusy(false);
    }
  };

  const handleExportApply = async () => {
    if (!exportProdDomain.trim() || !exportProdUpstream.trim()) return;
    setExportBusy(true);
    setExportError(null);
    setExportResult(null);
    try {
      const path = await api.exportNginxConfigToProject(
        initialDomain,
        exportProdDomain.trim(),
        exportProdUpstream.trim(),
      );
      setExportResult(path);
    } catch (e: any) {
      setExportError(String(e));
    } finally {
      setExportBusy(false);
    }
  };

  const openExportModal = () => {
    if (initialDomain.endsWith(".test") || initialDomain.endsWith(".local")) {
      const base = initialDomain.replace(/\.(test|local)$/, "");
      if (!exportProdDomain) setExportProdDomain(`${base}.example.com`);
    }
    if (!exportProdUpstream || exportProdUpstream === "http://127.0.0.1:3000") {
      setExportProdUpstream(upstream);
    }
    setExportResult(null);
    setExportError(null);
    setExportOpen(true);
  };

  useEffect(() => {
    setDomain(initialDomain);
    setUpstream(initialUpstream);
    setAdvancedConfig(initialAdvancedConfig || FULL_TEMPLATE);
    setProjectPath(initialProjectPath);
    setRunCommand(initialRunCommand);
  }, [initialDomain, initialUpstream, initialAdvancedConfig, initialProjectPath, initialRunCommand]);

  const handleClear = () => {
    if (confirm("Xóa trắng toàn bộ Nginx config?")) {
      setAdvancedConfig("");
    }
  };

  return (
    <div className="bg-surface-2 border border-surface-3/50 rounded-xl shadow-2xl overflow-hidden flex flex-col mb-10 transition-all duration-300 ring-1 ring-white/5">
      {/* Header Info */}
      <div className="p-6 border-b border-surface-3/30 bg-surface">
        <div className="flex items-center justify-between mb-5">
          <h2 className="text-xl font-bold text-text flex items-center gap-2">
            <svg className="w-5 h-5 text-accent" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" /></svg>
            {isEditing ? i18n.t("editorTitleEdit", { domain: initialDomain }) : i18n.t("editorTitleNew")}
          </h2>
          <div className="flex gap-3">
            <button
              type="button"
              onClick={() => { setImportError(null); setImportOpen(true); }}
              disabled={loading}
              className="px-3 py-2 text-xs font-semibold rounded-lg text-text-muted hover:text-accent hover:bg-accent/10 border border-surface-3 hover:border-accent/30 transition-all disabled:opacity-50 flex items-center gap-1.5"
              title="Dán nội dung .conf từ prod để tự động convert sang dev">
              <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
              </svg>
              {i18n.t("editorBtnImport")}
            </button>
            <button
              type="button"
              onClick={() => handleValidate()}
              disabled={loading || validateBusy || !advancedConfig.trim()}
              className="px-3 py-2 text-xs font-semibold rounded-lg text-text-muted hover:text-accent hover:bg-accent/10 border border-surface-3 hover:border-accent/30 transition-all disabled:opacity-50 flex items-center gap-1.5"
              title="Chạy nginx -t để kiểm tra syntax của advanced config">
              {validateBusy ? (
                <span className="w-3.5 h-3.5 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
              ) : (
                <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
              )}
              {i18n.t("editorBtnValidate")}
            </button>
            {isEditing && projectPath.trim() && (
              <button
                type="button"
                onClick={openExportModal}
                disabled={loading}
                className="px-3 py-2 text-xs font-semibold rounded-lg text-text-muted hover:text-accent hover:bg-accent/10 border border-surface-3 hover:border-accent/30 transition-all disabled:opacity-50 flex items-center gap-1.5"
                title="Xuất config ra thư mục dự án để dùng cho prod">
                <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                </svg>
                {i18n.t("editorBtnExport")}
              </button>
            )}
            {isEditing && (
              <button
                type="button"
                onClick={onCancel}
                disabled={loading}
                className="px-4 py-2 text-sm font-semibold rounded-lg text-text-muted hover:text-white hover:bg-surface-3 transition-colors disabled:opacity-50"
              >
                {i18n.t("editorBtnCancelEdit")}
              </button>
            )}
            <button
              onClick={() => onSave(domain, upstream, advancedConfig, projectPath, runCommand)}
              disabled={loading || !domain.trim() || !upstream.trim()}
              className="px-6 py-2 text-sm font-bold rounded-lg bg-accent text-white hover:bg-accent-hover active:scale-95 transition-all shadow-lg shadow-accent/20 disabled:opacity-50 disabled:active:scale-100 flex items-center gap-2"
            >
              {loading ? (
                <span className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              ) : (
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" /></svg>
              )}
              {isEditing ? i18n.t("editorBtnUpdate") : i18n.t("editorBtnCreate")}
            </button>
          </div>
        </div>

        <div className="flex flex-col md:flex-row gap-5">
          <div className="flex-1">
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-2 uppercase">{i18n.t("editorLabelDomain")}</label>
            <div className="relative">
              <span className="absolute left-3.5 top-2.5 text-text-muted/50">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" /></svg>
              </span>
              <input
                type="text"
                value={domain}
                onChange={(e) => setDomain(e.target.value)}
                placeholder="api.myapp.test"
                className="w-full pl-10 pr-4 py-2 rounded-lg bg-surface-3/30 border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 transition-all"
              />
            </div>
          </div>
          <div className="flex-1">
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-2 uppercase">{i18n.t("editorLabelUpstream")}</label>
            <div className="relative">
              <span className="absolute left-3.5 top-2.5 text-text-muted/50">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" /></svg>
              </span>
              <input
                type="text"
                value={upstream}
                onChange={(e) => setUpstream(e.target.value)}
                placeholder="http://127.0.0.1:3000"
                className="w-full pl-10 pr-4 py-2 rounded-lg bg-surface-3/30 border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 transition-all"
              />
            </div>
          </div>
        </div>

        <div className="flex flex-col md:flex-row gap-5 mt-5">
          <div className="flex-1">
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-2 uppercase">
              {i18n.t("editorLabelProjectPath")}
            </label>
            <div className="relative">
              <span className="absolute left-3.5 top-2.5 text-text-muted/50">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" /></svg>
              </span>
              <input
                type="text"
                value={projectPath}
                onChange={(e) => setProjectPath(e.target.value)}
                placeholder="C:\Users\you\Code\my-app"
                className="w-full pl-10 pr-4 py-2 rounded-lg bg-surface-3/30 border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 transition-all"
              />
            </div>
          </div>
          <div className="flex-1">
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-2 uppercase">
              {i18n.t("editorLabelRunCommand")}
            </label>
            <div className="relative">
              <span className="absolute left-3.5 top-2.5 text-text-muted/50">
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" /></svg>
              </span>
              <input
                type="text"
                value={runCommand}
                onChange={(e) => setRunCommand(e.target.value)}
                placeholder="npm run dev"
                disabled={!projectPath.trim()}
                className="w-full pl-10 pr-4 py-2 rounded-lg bg-surface-3/30 border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 transition-all disabled:opacity-50"
              />
            </div>
          </div>
        </div>
      </div>

      {(validateError || validateOk) && (
        <div className="px-6 py-3 border-b border-surface-3/30 bg-surface">
          {validateError && (
            <div className="flex items-start gap-2 text-xs text-danger bg-danger/10 border border-danger/20 rounded-lg p-2.5 font-mono whitespace-pre-wrap">
              <svg className="w-4 h-4 mt-0.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01M4.93 4.93a10 10 0 1014.14 0 10 10 0 00-14.14 0z" />
              </svg>
              <div className="flex-1 min-w-0 break-words">{validateError}</div>
              <button onClick={() => setValidateError(null)} className="text-danger/70 hover:text-danger shrink-0">✕</button>
            </div>
          )}
          {validateOk && !validateError && (
            <div className="flex items-center gap-2 text-xs text-success bg-success/10 border border-success/20 rounded-lg p-2.5 font-mono">
              <svg className="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              <div className="flex-1 min-w-0 break-words whitespace-pre-wrap">{validateOk}</div>
              <button onClick={() => setValidateOk(null)} className="text-success/70 hover:text-success shrink-0">✕</button>
            </div>
          )}
        </div>
      )}

      {/* Editor View */}
      <div className="flex flex-col h-[500px]">
        {/* Code Editor */}
        <div className="flex-1 bg-[#1e1e1e] flex flex-col relative w-full h-full min-h-[400px]">
          <div className="h-10 bg-[#252526] border-b border-[#333] flex items-center px-4 justify-between select-none">
            <div className="flex items-center gap-3">
              <div className="flex gap-1.5">
                <div className="w-3 h-3 rounded-full bg-red-500/80"></div>
                <div className="w-3 h-3 rounded-full bg-yellow-500/80"></div>
                <div className="w-3 h-3 rounded-full bg-green-500/80"></div>
              </div>
              <span className="text-xs font-mono text-gray-400">server_advanced.conf</span>
            </div>
            <button 
              onClick={handleClear}
              className="text-[10px] uppercase font-bold text-gray-500 hover:text-red-400 transition-colors"
            >
              Làm mới (Clear)
            </button>
          </div>
          
          <div className="flex-1 w-full h-full relative">
            <Editor
              height="100%"
              width="100%"
              defaultLanguage="ini"
              theme="vs-dark"
              value={advancedConfig}
              onChange={(value) => setAdvancedConfig(value || "")}
              options={{
                minimap: { enabled: false },
                fontSize: 13,
                fontFamily: "'JetBrains Mono', 'Fira Code', 'Courier New', monospace",
                lineHeight: 1.5,
                padding: { top: 16, bottom: 16 },
                scrollBeyondLastLine: false,
                smoothScrolling: true,
                cursorBlinking: "smooth",
                scrollbar: {
                    useShadows: false,
                    verticalHasArrows: false,
                    horizontalHasArrows: false,
                    vertical: 'visible',
                    horizontal: 'visible',
                }
              }}
            />
          </div>
        </div>
      </div>

      {importOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={() => setImportOpen(false)}>
          <div className="bg-surface-2 border border-surface-3 rounded-2xl shadow-2xl p-6 w-full max-w-2xl mx-4" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-bold text-text mb-2">{i18n.t("editorImportTitle")}</h3>
            <p className="text-xs text-text-muted mb-4">
              Dán nội dung file <code className="bg-surface-3/30 px-1 py-0.5 rounded font-mono">.conf</code> của bạn. Tool sẽ tự strip SSL/listen/server_name và rewrite <code className="bg-surface-3/30 px-1 py-0.5 rounded font-mono">proxy_pass</code> thành <code className="bg-surface-3/30 px-1 py-0.5 rounded font-mono">$UPSTREAM</code>.
            </p>
            <textarea
              value={importText}
              onChange={(e) => setImportText(e.target.value)}
              placeholder="server { ... }"
              rows={14}
              className="w-full px-3 py-2 rounded-lg bg-surface border border-surface-3 text-text font-mono text-xs focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 transition-all resize-y"
            />
            {importError && (
              <p className="mt-2 text-xs text-danger bg-danger/10 border border-danger/20 rounded-lg p-2.5 font-mono">{importError}</p>
            )}
            <div className="flex justify-end gap-2 mt-4">
              <button
                onClick={() => { setImportOpen(false); setImportError(null); }}
                className="px-4 py-2 rounded-lg text-sm font-semibold text-text-muted bg-surface-3/50 hover:bg-surface-3 transition-colors cursor-pointer">
                {i18n.t("btnCancel")}
              </button>
              <button
                onClick={handleImportApply}
                disabled={importBusy || !importText.trim()}
                className="px-5 py-2 rounded-lg text-sm font-bold text-white bg-accent hover:bg-accent-hover disabled:opacity-50 transition-all cursor-pointer">
                {importBusy ? i18n.t("editorImportProcessing") : i18n.t("editorImportBtn")}
              </button>
            </div>
          </div>
        </div>
      )}

      {exportOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={() => setExportOpen(false)}>
          <div className="bg-surface-2 border border-surface-3 rounded-2xl shadow-2xl p-6 w-full max-w-md mx-4" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-lg font-bold text-text mb-2">{i18n.t("editorExportTitle")}</h3>
            <p className="text-xs text-text-muted mb-4">
              Ghi file <code className="bg-surface-3/30 px-1 py-0.5 rounded font-mono">{`<project>/nginx/<prod-domain>.conf`}</code> đã strip SSL (để Certbot tự thêm trên prod).
            </p>
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-1.5 uppercase">{i18n.t("editorExportLabelDomain")}</label>
            <input
              type="text"
              value={exportProdDomain}
              onChange={(e) => setExportProdDomain(e.target.value)}
              placeholder="myapp.example.com"
              className="w-full px-3 py-2 rounded-lg bg-surface border border-surface-3 text-text font-mono text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50 mb-3"
            />
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-1.5 uppercase">{i18n.t("editorExportLabelUpstream")}</label>
            <input
              type="text"
              value={exportProdUpstream}
              onChange={(e) => setExportProdUpstream(e.target.value)}
              placeholder="http://127.0.0.1:3000"
              className="w-full px-3 py-2 rounded-lg bg-surface border border-surface-3 text-text font-mono text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/50"
            />
            {exportResult && (
              <p className="mt-3 text-xs text-success bg-success/10 border border-success/20 rounded-lg p-2.5 font-mono break-all">
                ✓ Đã ghi: {exportResult}
              </p>
            )}
            {exportError && (
              <p className="mt-3 text-xs text-danger bg-danger/10 border border-danger/20 rounded-lg p-2.5 font-mono">{exportError}</p>
            )}
            <div className="flex justify-end gap-2 mt-4">
              <button
                onClick={() => setExportOpen(false)}
                className="px-4 py-2 rounded-lg text-sm font-semibold text-text-muted bg-surface-3/50 hover:bg-surface-3 transition-colors cursor-pointer">
                {i18n.t("editorBtnClose")}
              </button>
              <button
                onClick={handleExportApply}
                disabled={exportBusy || !exportProdDomain.trim() || !exportProdUpstream.trim()}
                className="px-5 py-2 rounded-lg text-sm font-bold text-white bg-accent hover:bg-accent-hover disabled:opacity-50 transition-all cursor-pointer">
                {exportBusy ? i18n.t("editorExportWriting") : i18n.t("editorExportBtn")}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
