pub mod api;
pub mod config;
pub mod core;
pub mod dns;
pub mod logging;
pub mod protocols;
pub mod rules;
pub mod system;
pub mod tun;
pub mod utils;

#[cfg(feature = "gui")]
pub mod gui;

pub use config::{Config, ProxyMode};
pub use core::{ProxyServer, Result as ProxyResult};

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");
