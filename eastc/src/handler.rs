use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Local};
use east_core::{
    bootstrap::Bootstrap, byte_buf::ByteBuf, context::Context, handler::Handler, message::Msg,
    types::TypesEnum,
};
use tokio::{net::TcpStream, spawn, sync::{RwLock, self, broadcast::Sender}, time, select};

use crate::{
    config,
    forward::{decoder::ForwardDecoder, encoder::ForwardEncoder, handler::ForwardHandler},
    key,
};


#[derive(Clone)]
pub struct AgentHandler {
    channel_map: Arc<RwLock<HashMap<u64, Context<Vec<u8>>>>>,
    shutdown_heart:Sender<()>,
    last_time: Arc<RwLock<DateTime<Local>>>,
}

impl AgentHandler {
    pub fn new() -> Self {
      let (tx,_)=sync::broadcast::channel(64);
        AgentHandler {
            channel_map: Arc::new(RwLock::new(HashMap::new())),
            shutdown_heart:tx,
            last_time:Arc::new(RwLock::new(Local::now())),
        }
    }

    async fn auth(&mut self, ctx: &Context<Msg>, _msg: Msg) {
        log::debug!("start heartbeat thread");
        let ctx=ctx.clone();
        let mut sub=self.shutdown_heart.subscribe();
        let this=self.clone();
       
        spawn(async move{
          loop{
            let delay=Local::now()-*this.last_time.read().await;
            let seconds=delay.num_seconds();
            if seconds>60{
                log::debug!("expired heartbeat");
                ctx.close().await;
                return
            }
            select!{
              _=sub.recv()=>{
                log::debug!("exit heartbeat thread");
                return
              },
              _=async{
                let msg=Msg::new(TypesEnum::Heartbeat, vec![]);
                ctx.write(msg).await.unwrap_or_else(|e|log::warn!("{:?}",e));
              }=>{
                time::sleep(time::Duration::from_secs(10)).await;
              }
            }
          }
        });
        
    }

    async fn heartbeat(&mut self,_ctx:&Context<Msg>,_msg:Msg){
      let mut last_time=self.last_time.write().await;
      *last_time=Local::now();
    }

    async fn proxy_open(self, ctx: Context<Msg>, msg: Msg) {
        let bytes = msg.data;
        let mut bf = ByteBuf::new_from(&bytes[..]);
        let host = bf.read_string_with_u8_be_len();
        let port = bf.read_u16_be();
        let addr = format!("{}:{}", host, port).to_string();
        let id = bf.read_u64_be();

        let stream = TcpStream::connect(addr).await;
        match stream {
            Ok(stream) => {
                let addr = stream.peer_addr().unwrap();
                log::info!("connect {}", addr);
                let handler = ForwardHandler::new(ctx.clone(), id, self.channel_map.clone());
                let mut boot =
                    Bootstrap::build(stream, addr, ForwardEncoder {}, ForwardDecoder {}, handler);
                boot.capacity(1024);

                let result = boot.run().await;
                if let Err(e) = result {
                    log::error!("{:?}", e);
                }
            }
            Err(e) => {
                log::error!("{:?}", e);
            }
        }
        let mut bf = ByteBuf::new_with_capacity(0);
        bf.write_u64_be(id).unwrap_or_else(|err| {
            log::error!("{}", err);
            0
        });
        let msg = Msg::new(TypesEnum::ProxyClose, bf.available_bytes().to_vec());
        ctx.write(msg).await.unwrap_or_else(|err| {
            log::warn!("{}", err);
        });
    }
    async fn proxy_forward(&mut self, _ctx: &Context<Msg>, msg: Msg) {
        let bytes = msg.data;
        let mut bf = ByteBuf::new_from(&bytes[..]);
        let id = bf.read_u64_be();
        let mut buf = vec![0u8; bf.readable_bytes()];
        bf.read_bytes(&mut buf);
        if let Some(channel_ctx) = self.channel_map.read().await.get(&id) {
            channel_ctx.write(buf).await.unwrap_or_else(|err| {
                log::warn!("{}", err);
            });
        } else {
            log::warn!("not found channel id {}", id);
        }
    }
    async fn proxy_close(&mut self, _ctx: &Context<Msg>, msg: Msg) {
        let mut bf = ByteBuf::new_from(&msg.data);
        let id = bf.read_u64_be();
        if let Some(channel_ctx) = self.channel_map.read().await.get(&id) {
            channel_ctx.close().await;
        }
        self.channel_map.write().await.remove(&id);
    }
}

#[async_trait::async_trait]
impl Handler<Msg> for AgentHandler {
    async fn read(&mut self, ctx: &Context<Msg>, msg: Msg) {
        match msg.msg_type {
            TypesEnum::Auth => self.auth(ctx, msg).await,
            TypesEnum::ProxyOpen => {
                let this = self.clone();
                spawn(this.proxy_open(ctx.clone(), msg));
            }
            TypesEnum::ProxyForward => self.proxy_forward(ctx, msg).await,
            TypesEnum::ProxyClose => self.proxy_close(ctx, msg).await,
            TypesEnum::Heartbeat => self.heartbeat(ctx,msg).await,
            _ => {}
        };
    }
    async fn active(&mut self, ctx: &Context<Msg>) {
        log::info!("connected {:?}", ctx.addr());
        let conf = Arc::clone(&config::CONF);
        let id = conf.id.clone();
        match key::encrypt(id) {
            Ok(data) => {
                let msg = Msg::new(TypesEnum::Auth, data);
                ctx.write(msg).await.unwrap_or_else(|err| {
                    log::warn!("{}", err);
                });
            }
            Err(e) => {
                log::error!("public key loading failed {:?}", e);
                ctx.close().await;
            }
        }
    }
    async fn close(&mut self, ctx: &Context<Msg>) {
        log::info!("closed {:?} ", ctx.addr());
        self.shutdown_heart.send(()).unwrap_or_else(|e|{log::error!("{:?}",e);0});
        for (k, v) in self.channel_map.read().await.iter() {
            log::debug!("close channel id {}", k);
            v.close().await;
        }
        self.channel_map.write().await.clear();
    }
}
