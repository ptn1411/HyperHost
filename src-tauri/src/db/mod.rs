use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamedTunnelConfig {
    pub id: Option<i64>,
    pub tunnel_name: String,
    pub tunnel_id: Option<String>,
    pub credentials_path: Option<String>,
    pub hostname: String,
    pub upstream: String,
    pub enabled: bool,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DomainConfig {
    pub id: Option<i64>,
    pub domain: String,
    pub upstream: String,
    pub enabled: bool,
    pub cert_expiry: Option<String>,
    pub created_at: Option<String>,
    pub advanced_config: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct DomainStatus {
    pub config: DomainConfig,
    pub cert_valid: bool,
    pub cert_expiry: Option<String>,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn open(db_path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS domains (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                domain      TEXT NOT NULL UNIQUE,
                upstream    TEXT NOT NULL,
                enabled     INTEGER NOT NULL DEFAULT 1,
                cert_pem    TEXT,
                key_pem     TEXT,
                cert_expiry TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at  TEXT NOT NULL DEFAULT (datetime('now')),
                advanced_config TEXT
            );
            CREATE TABLE IF NOT EXISTS named_tunnels (
                id               INTEGER PRIMARY KEY AUTOINCREMENT,
                tunnel_name      TEXT NOT NULL UNIQUE,
                tunnel_id        TEXT,
                credentials_path TEXT,
                hostname         TEXT NOT NULL,
                upstream         TEXT NOT NULL,
                enabled          INTEGER NOT NULL DEFAULT 1,
                created_at       TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )?;

        // Auto-migrate schema for existing databases
        let _ = conn.execute("ALTER TABLE domains ADD COLUMN advanced_config TEXT", []);

        tracing::info!("Database opened at {}", db_path.display());
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn upsert_domain(
        &self,
        cfg: &DomainConfig,
        cert_pem: &str,
        key_pem: &str,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO domains (domain, upstream, enabled, cert_pem, key_pem, cert_expiry, advanced_config)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(domain) DO UPDATE SET
                upstream    = excluded.upstream,
                enabled     = excluded.enabled,
                cert_pem    = excluded.cert_pem,
                key_pem     = excluded.key_pem,
                cert_expiry = excluded.cert_expiry,
                advanced_config = excluded.advanced_config,
                updated_at  = datetime('now')",
            params![
                cfg.domain,
                cfg.upstream,
                cfg.enabled as i32,
                cert_pem,
                key_pem,
                cfg.cert_expiry,
                cfg.advanced_config.as_deref(),
            ],
        )?;
        Ok(())
    }

    pub fn list_domains(&self) -> anyhow::Result<Vec<DomainConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, domain, upstream, enabled, cert_expiry, created_at, advanced_config FROM domains ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DomainConfig {
                id: Some(row.get(0)?),
                domain: row.get(1)?,
                upstream: row.get(2)?,
                enabled: row.get::<_, i32>(3)? != 0,
                cert_expiry: row.get(4)?,
                created_at: row.get(5)?,
                advanced_config: row.get(6)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn list_enabled_domains(&self) -> anyhow::Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT domain FROM domains WHERE enabled = 1")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn remove_domain(&self, domain: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM domains WHERE domain = ?1", params![domain])?;
        Ok(())
    }

    pub fn toggle_domain(&self, domain: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE domains SET enabled = 1 - enabled, updated_at = datetime('now') WHERE domain = ?1",
            params![domain],
        )?;
        let new_state: bool = conn.query_row(
            "SELECT enabled FROM domains WHERE domain = ?1",
            params![domain],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        )?;
        Ok(new_state)
    }

    // ── Named Tunnel methods ──

    pub fn insert_named_tunnel(&self, cfg: &NamedTunnelConfig) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO named_tunnels (tunnel_name, hostname, upstream) VALUES (?1, ?2, ?3)",
            params![cfg.tunnel_name, cfg.hostname, cfg.upstream],
        )?;
        Ok(())
    }

    pub fn list_named_tunnels(&self) -> anyhow::Result<Vec<NamedTunnelConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tunnel_name, tunnel_id, credentials_path, hostname, upstream, enabled, created_at
             FROM named_tunnels ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(NamedTunnelConfig {
                id: Some(row.get(0)?),
                tunnel_name: row.get(1)?,
                tunnel_id: row.get(2)?,
                credentials_path: row.get(3)?,
                hostname: row.get(4)?,
                upstream: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
            })
        })?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }

    pub fn get_named_tunnel(&self, name: &str) -> anyhow::Result<Option<NamedTunnelConfig>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tunnel_name, tunnel_id, credentials_path, hostname, upstream, enabled, created_at
             FROM named_tunnels WHERE tunnel_name = ?1",
        )?;
        let mut rows = stmt.query_map(params![name], |row| {
            Ok(NamedTunnelConfig {
                id: Some(row.get(0)?),
                tunnel_name: row.get(1)?,
                tunnel_id: row.get(2)?,
                credentials_path: row.get(3)?,
                hostname: row.get(4)?,
                upstream: row.get(5)?,
                enabled: row.get::<_, i32>(6)? != 0,
                created_at: row.get(7)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_named_tunnel_credentials(
        &self,
        name: &str,
        tunnel_id: &str,
        credentials_path: &str,
    ) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE named_tunnels SET tunnel_id = ?1, credentials_path = ?2 WHERE tunnel_name = ?3",
            params![tunnel_id, credentials_path, name],
        )?;
        Ok(())
    }

    pub fn remove_named_tunnel(&self, name: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM named_tunnels WHERE tunnel_name = ?1",
            params![name],
        )?;
        Ok(())
    }

    pub fn toggle_named_tunnel(&self, name: &str) -> anyhow::Result<bool> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE named_tunnels SET enabled = 1 - enabled WHERE tunnel_name = ?1",
            params![name],
        )?;
        let new_state: bool = conn.query_row(
            "SELECT enabled FROM named_tunnels WHERE tunnel_name = ?1",
            params![name],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        )?;
        Ok(new_state)
    }
}
