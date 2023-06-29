#[macro_use]
extern crate lazy_static; 
extern crate east_core;

mod log_conf;
mod config;
mod agent;
mod handler;
mod forward;
mod key;

use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    log_conf::init();

    let version: &'static str = env!("CARGO_PKG_VERSION");
    log::info!("version: {}", version);
    let author: &'static str = env!("CARGO_PKG_AUTHORS");
    log::info!("author: {}", author);
    agent::run().await;
    Ok(())
}
