import { useEffect, useState } from "react";
import { api, NamedTunnelConfig } from "../lib/tauri";
import { i18n } from "../translation";

export function NamedTunnelPanel() {
  const [tunnels, setTunnels] = useState<NamedTunnelConfig[]>([]);
  const [runningMap, setRunningMap] = useState<Record<string, boolean>>({});
  const [isLoggedIn, setIsLoggedIn] = useState<boolean | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  // Add form state
  const [form, setForm] = useState({ tunnelName: "", hostname: "", upstream: "http://127.0.0.1:3000" });
  const [showForm, setShowForm] = useState(false);

  const refresh = async () => {
    try {
      const [list, loggedIn] = await Promise.all([
        api.listNamedTunnels(),
        api.cloudflareLoginStatus(),
      ]);
      setTunnels(list);
      setIsLoggedIn(loggedIn);

      // Check running status for each tunnel
      const map: Record<string, boolean> = {};
      await Promise.all(
        list.map(async (t) => {
          map[t.tunnel_name] = await api.namedTunnelRunning(t.tunnel_name);
        })
      );
      setRunningMap(map);
    } catch (e) {
      console.error(e);
    }
  };

  useEffect(() => {
    refresh();
  }, []);

  const showSuccess = (msg: string) => {
    setSuccessMsg(msg);
    setTimeout(() => setSuccessMsg(null), 3000);
  };

  const handleLogin = async () => {
    setLoading(true);
    setError(null);
    try {
      await api.cloudflareLogin();
      await refresh();
      showSuccess("Đăng nhập Cloudflare thành công");
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleAdd = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await api.addNamedTunnel(form.tunnelName.trim(), form.hostname.trim(), form.upstream.trim());
      setForm({ tunnelName: "", hostname: "", upstream: "http://127.0.0.1:3000" });
      setShowForm(false);
      await refresh();
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleProvision = async (name: string) => {
    setLoading(true);
    setError(null);
    try {
      await api.provisionNamedTunnel(name);
      await refresh();
      showSuccess(`Tunnel "${name}" đã được tạo trên Cloudflare`);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleToggleRun = async (t: NamedTunnelConfig) => {
    setLoading(true);
    setError(null);
    try {
      if (runningMap[t.tunnel_name]) {
        await api.stopNamedTunnel(t.tunnel_name);
      } else {
        await api.startNamedTunnel(t.tunnel_name);
      }
      await refresh();
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const handleRemove = async (name: string) => {
    setLoading(true);
    setError(null);
    try {
      await api.removeNamedTunnel(name);
      await refresh();
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const isProvisioned = (t: NamedTunnelConfig) => !!t.tunnel_id;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between pb-3 border-b border-surface-3/30">
        <div>
          <h2 className="text-xl font-bold text-text">{i18n.t("namedTunnelTitle")}</h2>
          <p className="text-text-muted text-sm mt-0.5">
            {i18n.t("namedTunnelDesc")}
          </p>
        </div>

        {/* Login status */}
        {isLoggedIn === false ? (
          <button
            onClick={handleLogin}
            disabled={loading}
            className="flex items-center gap-2 px-4 py-2 rounded-lg bg-orange-500/10 text-orange-400 border border-orange-500/20 hover:bg-orange-500/20 text-sm font-semibold transition-all cursor-pointer disabled:opacity-50">
            <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
            </svg>
            {i18n.t("namedTunnelLogin")}
          </button>
        ) : isLoggedIn === true ? (
          <div className="flex items-center gap-2 px-4 py-2 rounded-lg bg-surface-2 border border-surface-3 text-text-muted text-sm">
            <span className="text-success">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
              </svg>
            </span>
            {i18n.t("namedTunnelConnected")}
          </div>
        ) : null}
      </div>

      {/* Info box */}
      <div className="p-4 rounded-xl bg-blue-500/5 border border-blue-500/20 text-blue-400 text-sm">
        <p className="font-semibold mb-1">{i18n.t("namedTunnelRequirements")}</p>
        <ul className="list-disc list-inside space-y-0.5 text-blue-400/80 text-xs">
          <li>{i18n.t("namedTunnelReqOwn")}</li>
          <li>{i18n.t("namedTunnelReqLogin")}</li>
          <li>{i18n.t("namedTunnelReqEach")}</li>
        </ul>
      </div>

      {/* Error / Success */}
      {error && (
        <div className="p-3 rounded-xl bg-danger/10 border border-danger/30 text-danger text-xs font-mono flex items-start gap-2">
          <span className="shrink-0 mt-0.5">✕</span>
          <span className="flex-1">{error}</span>
          <button onClick={() => setError(null)} className="cursor-pointer hover:opacity-70">✕</button>
        </div>
      )}
      {successMsg && (
        <div className="p-3 rounded-xl bg-success/10 border border-success/30 text-success text-sm flex items-center gap-2">
          <svg className="w-4 h-4 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M5 13l4 4L19 7" />
          </svg>
          {successMsg}
        </div>
      )}

      {/* Add Tunnel Button */}
      {!showForm && (
        <button
          onClick={() => setShowForm(true)}
          className="flex items-center gap-2 px-4 py-2.5 rounded-lg bg-accent text-white text-sm font-semibold hover:bg-accent-hover transition-all cursor-pointer shadow-md shadow-accent/20">
          <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
          </svg>
          {i18n.t("namedTunnelAdd")}
        </button>
      )}

      {/* Add Form */}
      {showForm && (
        <form
          onSubmit={handleAdd}
          className="p-5 rounded-xl bg-surface-2 border border-surface-3/50 space-y-4">
          <h3 className="font-semibold text-text">{i18n.t("namedTunnelAddNew")}</h3>

          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <div>
              <label className="block text-xs font-semibold text-text-muted uppercase tracking-wide mb-1.5">
                {i18n.t("namedTunnelLabelName")}
              </label>
              <input
                type="text"
                value={form.tunnelName}
                onChange={(e) => setForm({ ...form, tunnelName: e.target.value })}
                placeholder="my-tunnel"
                required
                className="w-full px-3 py-2.5 rounded-lg bg-surface border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 transition-all"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-text-muted uppercase tracking-wide mb-1.5">
                {i18n.t("namedTunnelLabelHostname")}
              </label>
              <input
                type="text"
                value={form.hostname}
                onChange={(e) => setForm({ ...form, hostname: e.target.value })}
                placeholder="app.yourdomain.com"
                required
                className="w-full px-3 py-2.5 rounded-lg bg-surface border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 transition-all"
              />
            </div>
            <div>
              <label className="block text-xs font-semibold text-text-muted uppercase tracking-wide mb-1.5">
                {i18n.t("namedTunnelLabelUpstream")}
              </label>
              <input
                type="text"
                value={form.upstream}
                onChange={(e) => setForm({ ...form, upstream: e.target.value })}
                placeholder="http://127.0.0.1:3000"
                required
                className="w-full px-3 py-2.5 rounded-lg bg-surface border border-surface-3 text-text font-mono text-sm placeholder:text-text-muted/40 focus:outline-none focus:border-accent focus:ring-2 focus:ring-accent/20 transition-all"
              />
            </div>
          </div>

          <div className="flex gap-3">
            <button
              type="submit"
              disabled={loading}
              className="px-5 py-2 rounded-lg bg-accent text-white text-sm font-semibold hover:bg-accent-hover transition-all cursor-pointer disabled:opacity-50">
              {i18n.t("namedTunnelBtnAdd")}
            </button>
            <button
              type="button"
              onClick={() => setShowForm(false)}
              className="px-5 py-2 rounded-lg bg-surface-3 text-text-muted text-sm font-semibold hover:text-text transition-all cursor-pointer">
              {i18n.t("namedTunnelBtnCancel")}
            </button>
          </div>
        </form>
      )}

      {/* Tunnel List */}
      {tunnels.length === 0 ? (
        <div className="text-center py-16 bg-surface-2 border border-surface-3/30 rounded-xl border-dashed">
          <svg className="w-10 h-10 mx-auto mb-3 text-text-muted/30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
              d="M3 15a4 4 0 004 4h9a5 5 0 10-.1-9.999 5.002 5.002 0 10-9.78 2.096A4.001 4.001 0 003 15z" />
          </svg>
          <p className="text-text-muted font-medium">{i18n.t("namedTunnelEmpty")}</p>
          <p className="text-text-muted/60 text-sm mt-1">{i18n.t("namedTunnelEmptyDesc")}</p>
        </div>
      ) : (
        <div className="space-y-3">
          {tunnels.map((t) => {
            const running = runningMap[t.tunnel_name] ?? false;
            const provisioned = isProvisioned(t);
            return (
              <div key={t.tunnel_name}
                className="p-5 rounded-xl bg-surface-2 border border-surface-3/50 hover:border-accent/30 transition-all">
                <div className="flex items-center justify-between gap-4">
                  <div className="flex items-center gap-4 min-w-0">
                    {/* Running indicator */}
                    <span className={`w-2.5 h-2.5 rounded-full shrink-0 ${
                      running ? "bg-success animate-pulse shadow-success/50 shadow-sm" : "bg-surface-3"
                    }`} />

                    <div className="min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-mono font-semibold text-text">{t.tunnel_name}</span>
                        {provisioned ? (
                          <span className="text-xs px-2 py-0.5 rounded-full bg-success/10 text-success border border-success/20">
                            {i18n.t("namedTunnelProvisioned")}
                          </span>
                        ) : (
                          <span className="text-xs px-2 py-0.5 rounded-full bg-warning/10 text-warning border border-warning/20">
                            {i18n.t("namedTunnelNotProvisioned")}
                          </span>
                        )}
                        {running && (
                          <span className="text-xs px-2 py-0.5 rounded-full bg-success/10 text-success border border-success/20">
                            {i18n.t("namedTunnelRunning")}
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-2 mt-1 text-xs text-text-muted font-mono">
                        <span className="text-accent">{t.hostname}</span>
                        <span>→</span>
                        <span>{t.upstream}</span>
                      </div>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 shrink-0">
                    {!provisioned ? (
                      <button
                        onClick={() => handleProvision(t.tunnel_name)}
                        disabled={loading || !isLoggedIn}
                        title={!isLoggedIn ? i18n.t("namedTunnelNeedLogin") : i18n.t("namedTunnelProvisionTooltip")}
                        className="px-3 py-1.5 rounded-lg bg-orange-500/10 text-orange-400 border border-orange-500/20 hover:bg-orange-500/20 text-xs font-semibold transition-all cursor-pointer disabled:opacity-40 disabled:cursor-not-allowed">
                        {i18n.t("namedTunnelProvision")}
                      </button>
                    ) : (
                      <button
                        onClick={() => handleToggleRun(t)}
                        disabled={loading}
                        className={`px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer disabled:opacity-50 ${
                          running
                            ? "bg-danger/10 text-danger border border-danger/20 hover:bg-danger/20"
                            : "bg-success/10 text-success border border-success/20 hover:bg-success/20"
                        }`}>
                        {running ? i18n.t("namedTunnelStop") : i18n.t("namedTunnelStart")}
                      </button>
                    )}

                    <button
                      onClick={() => handleRemove(t.tunnel_name)}
                      disabled={loading}
                      className="p-1.5 rounded-lg text-text-muted hover:text-danger hover:bg-danger/10 transition-all cursor-pointer disabled:opacity-50">
                      <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2}
                          d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </div>

                {provisioned && t.tunnel_id && (
                  <div className="mt-3 pt-3 border-t border-surface-3/30 text-xs font-mono text-text-muted/60">
                    Tunnel ID: {t.tunnel_id}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
