use std::{sync::Arc, net::SocketAddr};
use east_core::bootstrap::Bootstrap;
use east_core::message::msg_decoder::MsgDecoder;
use east_core::message::msg_encoder::MsgEncoder;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::Result;
use crate::config;
use crate::connection::ConnectionManager;
use crate::handler::ServerHandler;

pub async fn run(manager:ConnectionManager) -> Result<()> {
  let conf=Arc::clone(&config::CONF);
  let addr:SocketAddr=conf.server.bind.parse().unwrap();
  let listener=TcpListener::bind(addr).await.unwrap();
  log::info!("service startup -> {}",addr);
  loop{
      let (socket,addr)=listener.accept().await.unwrap();
      let manager=manager.clone();
      tokio::spawn(async move{
          process_socket(socket,addr,manager).await;
      });
  }
}

async fn process_socket(client:TcpStream,addr:SocketAddr,manager:ConnectionManager){
  let handler=ServerHandler::new(manager);
  let mut boot=Bootstrap::build(client,addr, MsgEncoder{}, MsgDecoder{}, handler);
  boot.capacity(1024);
  if let Err(e)=boot.run().await{
      log::error!("{:?}",e);
  }
}
