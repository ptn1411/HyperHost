import { useState, useEffect } from "react";
import Editor from "@monaco-editor/react";

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
            {isEditing ? `Chỉnh sửa Domain: ${initialDomain}` : "Tạo cấu hình Proxy Mới"}
          </h2>
          <div className="flex gap-3">
            {isEditing && (
              <button 
                type="button" 
                onClick={onCancel}
                disabled={loading}
                className="px-4 py-2 text-sm font-semibold rounded-lg text-text-muted hover:text-white hover:bg-surface-3 transition-colors disabled:opacity-50"
              >
                Hủy biên tập
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
              {isEditing ? "Cập nhật Config" : "Tạo Route"}
            </button>
          </div>
        </div>

        <div className="flex flex-col md:flex-row gap-5">
          <div className="flex-1">
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-2 uppercase">Local Domain Khách (ví dụ: myapp.test)</label>
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
            <label className="block text-[11px] font-bold tracking-wider text-text-muted mb-2 uppercase">Upstream Đích (ví dụ: http://127.0.0.1:8080)</label>
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
              Thư mục dự án (tùy chọn)
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
              Lệnh Run (tùy chọn)
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
    </div>
  );
}
