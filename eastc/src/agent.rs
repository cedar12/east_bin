use east_core::{bootstrap::Bootstrap, message::{msg_encoder::MsgEncoder, msg_decoder::MsgDecoder}};
use tokio::{ net::TcpStream, time};

use crate::{handler::AgentHandler, config};

const WAIT_TIME:u64=3000;

pub async fn run() {
    loop {
        let stream = TcpStream::connect(config::CONF.server.clone()).await;
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().unwrap();
                let mut boot=Bootstrap::build(
                    stream,
                    addr,
                    MsgEncoder {},
                    MsgDecoder {},
                    AgentHandler::new(),
                );
                boot.capacity(1024);
                let result = boot.run().await;
                if let Err(e) = result {
                    log::error!("{:}", e);
                }
            }
            Err(e) => {
                log::error!("{:?}", e)
            }
        }
        log::info!("waiting for reconnection");
        time::sleep(time::Duration::from_millis(WAIT_TIME)).await;
    }
}
