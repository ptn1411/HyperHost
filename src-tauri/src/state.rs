use crate::cert::ca::LocalCA;
use crate::db::Database;
use crate::nginx::NginxManager;
use crate::paths::AppPaths;
use std::sync::atomic::AtomicBool;

pub struct AppState {
    pub paths: AppPaths,
    pub db: Database,
    pub ca: LocalCA,
    pub nginx: NginxManager,
    /// If true, clicking the window X button hides to tray instead of quitting.
    pub minimize_to_tray: AtomicBool,
    #[cfg(feature = "gui")]
    pub cloudflared: crate::cloudflare::CloudflaredManager,
    #[cfg(feature = "gui")]
    pub named_tunnels: crate::cloudflare::named_tunnel::NamedTunnelManager,
}
