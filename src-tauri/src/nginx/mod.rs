pub mod config;
#[cfg(feature = "gui")]
pub mod tail;
mod manager;
pub use manager::NginxManager;
