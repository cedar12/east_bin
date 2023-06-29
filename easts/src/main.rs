#[macro_use]
extern crate lazy_static; 
extern crate east_core;


mod config;
mod log_conf;
mod server;
mod handler;
mod connection;
mod constant;
mod key;
mod agent;
mod forward;

use connection::ConnectionManager;
use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    log_conf::init();
    let version: &'static str = env!("CARGO_PKG_VERSION");
    log::info!("version: {}", version);
    let author: &'static str = env!("CARGO_PKG_AUTHORS");
    log::info!("author: {}", author);
    let manager=ConnectionManager::new();
    server::run(manager).await
}

