use crate::cert::ca::LocalCA;
use crate::db::Database;
use crate::nginx::NginxManager;
use crate::paths::AppPaths;

pub struct AppState {
    pub paths: AppPaths,
    pub db: Database,
    pub ca: LocalCA,
    pub nginx: NginxManager,
    #[cfg(feature = "gui")]
    pub cloudflared: crate::cloudflare::CloudflaredManager,
}
