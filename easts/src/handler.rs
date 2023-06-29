use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Local};
use east_core::byte_buf::ByteBuf;
use east_core::{
    bootstrap::Bootstrap, context::Context, handler::Handler, message::Msg, types::TypesEnum,
};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream};
use tokio::task::JoinHandle;
use tokio::{
    net::TcpListener,
    select, spawn,
    sync::{
        mpsc::{self, Sender},
        Mutex, RwLock,
    },
};

use crate::forward::decoder::ForwardDecoder;
use crate::forward::encoder::ForwardEncoder;
use crate::forward::handler::ForwardHandler;
use crate::forward::{match_addr, ForwardMsg};
use crate::{
    agent::{self, Forward},
    connection::ConnectionManager,
    constant, key,
};

lazy_static! {
    pub static ref LAST_ID: AtomicU32 = AtomicU32::new(1);
}

#[derive(Clone)]
pub struct ServerHandler {
    manager: ConnectionManager,
    id: Option<String>,
    forawrd_shutdown: Arc<RwLock<HashMap<u16, Arc<Mutex<Sender<()>>>>>>,
    channel_map: Arc<RwLock<HashMap<u64, Context<ForwardMsg>>>>,
    listen_task: HashMap<u16, Arc<Mutex<JoinHandle<()>>>>,
    is_closed: bool,
    last_time:Arc<RwLock<DateTime<Local>>>,
}

impl ServerHandler {
    pub fn new(manager: ConnectionManager) -> Self {
        Self {
            manager: manager,
            id: None,
            forawrd_shutdown: Arc::new(RwLock::new(HashMap::new())),
            channel_map: Arc::new(RwLock::new(HashMap::new())),
            listen_task: HashMap::new(),
            is_closed: false,
            last_time:Arc::new(RwLock::new(Local::now())),
        }
    }

    async fn listen(self, ctx: Context<Msg>, forward: Forward) {
        let (sender, mut receiver) = mpsc::channel::<()>(1);
        self.forawrd_shutdown
            .write()
            .await
            .insert(forward.bind_port, Arc::new(Mutex::new(sender)));
        let listen = TcpListener::bind(format!("0.0.0.0:{}", forward.bind_port)).await;
        log::info!("open forward listen port {}", forward.bind_port);
        match listen {
            Ok(listen) => loop {
                select! {
                  _=receiver.recv()=>{
                    log::info!("shutdown forward listen port {}",forward.bind_port);
                    return;
                  }
                  ret=listen.accept()=>{
                    let (mut stream,addr)=ret.unwrap();
                    if !match_addr(forward.whitelist.clone(),addr.to_string()){
                      log::warn!("ip->{:?},not in the whitelist list, not allowed\n{:?}",addr,forward.whitelist);
                      let _=stream.shutdown().await;
                    }else{
                      let id=LAST_ID.fetch_add(1,Ordering::Relaxed) as u64;
                      if (id as u32)==u32::MAX{
                        LAST_ID.store(1, Ordering::Relaxed);
                      }
                      let handler=ForwardHandler::new(ctx.clone(),id,forward.bind_port,self.channel_map.clone());
                      let mut forward_boot=Bootstrap::build(stream, addr,ForwardEncoder::new(forward.max_rate) , ForwardDecoder::new(forward.max_rate), handler);
                      forward_boot.capacity(1024);
                      
                      ctx.set_attribute(format!("{}_{}",constant::FORWARD_BOOT,id), Box::new(Arc::new(Mutex::new(forward_boot)))).await;
                      let mut bf=ByteBuf::new_with_capacity(0);
                      let host=forward.target_host.clone();
                      let port=forward.target_port;
                      bf.write_string_with_u8_be_len(host).unwrap_or_else(|e|{log::error!("{:?}",e);0});
                      bf.write_u16_be(port).unwrap_or_else(|e|{log::error!("{:?}",e);0});
                      bf.write_u64_be(id).unwrap_or_else(|e|{log::error!("{:?}",e);0});
                      let open_msg=Msg::new(TypesEnum::ProxyOpen,bf.available_bytes().to_vec());
                      ctx.write(open_msg).await.unwrap_or_else(|e|log::warn!("{:?}",e));
                    }

                  }
                }
            },
            Err(e) => {
                log::error!("port:{}  {:?}", forward.bind_port, e);
                ctx.close().await;
            }
        }
    }

    async fn check_health(&mut self,ctx:Context<Msg>){
      let mut this=self.clone();
      spawn(async move{
        loop{
          let diff=Local::now()-*this.last_time.read().await;
          if diff.num_seconds()>30{
            log::debug!("{:?} expired",this.id.clone().unwrap_or_default());
            ctx.close().await;
            this.close(&ctx).await;
            return
          }
          tokio::time::sleep(Duration::from_secs(10)).await;
        }
      });
    }

    async fn auth(&mut self, ctx: &Context<Msg>, msg: Msg) {
        let id = key::decrypt(msg.data);
        match id {
            Ok(id) => {
                let agent = agent::get_agent(id.clone());
                match agent {
                    Some(agent) => {
                        // 有新连接，关闭旧连接
                        match self.manager.get(id.clone()).await {
                          Some(old_ctx) => {
                            log::info!("close old connection {}", id.clone());
                              old_ctx.close().await;
                          }
                          None => {}
                        }
                        let _ = self.id.insert(id.clone());
                        log::info!("{} connected", id.clone());
                        
                        self.manager.add(id.clone(), ctx.clone()).await;
                        let msg = Msg::new(TypesEnum::Auth, vec![]);
                        ctx.write(msg)
                            .await
                            .unwrap_or_else(|e| log::warn!("{:?}", e));
                        self.check_health(ctx.clone()).await;
                        for ele in agent.forward {
                            if self.is_closed {
                                return;
                            }
                            if !ele.enable {
                                continue;
                            }
                            match self.forawrd_shutdown.read().await.get(&ele.bind_port) {
                                Some(shutdown) => {
                                    log::debug!("shutdown {} forward port", id);
                                    shutdown
                                        .lock()
                                        .await
                                        .send(())
                                        .await
                                        .unwrap_or_else(|e| log::warn!("{:?}", e));
                                }
                                None => {}
                            }
                            let this = self.clone();
                            let join = spawn(this.listen(ctx.clone(), ele.clone()));
                            self.listen_task
                                .insert(ele.bind_port, Arc::new(Mutex::new(join)));
                        }
                    }
                    None => {
                        log::warn!("[{}]identification that does not exist", id);

                        ctx.close().await;
                    }
                }
            }
            Err(_) => {
                ctx.close().await;
            }
        }
    }

    async fn proxy_open(&mut self, ctx: &Context<Msg>, msg: Msg) {
        let mut bf = ByteBuf::new_from(&msg.data);
        let fid = bf.read_u64_be();
        let stream = ctx
            .get_attribute(format!("{}_{}", constant::FORWARD_BOOT, fid))
            .await;
        let stream = stream.lock().await;
        if let Some(boot) = stream.downcast_ref::<Arc<
            Mutex<Bootstrap<ForwardEncoder, ForwardDecoder, ForwardHandler, ForwardMsg, TcpStream>>,
        >>() {
            let boot = Arc::clone(boot);
            ctx.remove_attribute(format!("{}_{}", constant::FORWARD_BOOT, fid))
                .await;
            spawn(async move {
                let ret = boot.lock().await.run().await;
                if let Err(e) = ret {
                    log::warn!("{:?}", e);
                }
                log::debug!("id->{} closed", fid);
            });
        } else {
            log::debug!("open {} none", fid);
        }
    }
    async fn proxy_forward(&mut self, ctx: &Context<Msg>, msg: Msg) {
        let mut bf = ByteBuf::new_from(&msg.data);
        let id = bf.read_u64_be();

        match self.channel_map.read().await.get(&id) {
            Some(ctx) => {
                let mut buf = vec![0u8; bf.readable_bytes()];
                bf.read_bytes(&mut buf);
                ctx.write(ForwardMsg { buf: buf })
                    .await
                    .unwrap_or_else(|e| log::warn!("{:?}", e));
            }
            None => {
                log::debug!("{} no connection", id);
                let mut bf = ByteBuf::new_with_capacity(0);
                bf.write_u64_be(id).unwrap_or_else(|e| {
                    log::warn!("{:?}", e);
                    0
                });
                let msg = Msg::new(TypesEnum::ProxyClose, bf.available_bytes().to_vec());
                ctx.write(msg)
                    .await
                    .unwrap_or_else(|e| log::warn!("{:?}", e));
            }
        }
    }
    async fn proxy_close(&mut self, _ctx: &Context<Msg>, msg: Msg) {
        let mut bf = ByteBuf::new_from(&msg.data);
        let id = bf.read_u64_be();
        match self.channel_map.read().await.get(&id) {
            Some(ctx) => {
                ctx.close().await;
                log::debug!("close connection {} ", id);
            }
            None => {
                log::debug!("close no connection {}", id)
            }
        }
    }
    async fn heartbeat(&mut self,ctx:&Context<Msg>,_msg:Msg){
      let mut last_time=self.last_time.write().await;
      *last_time=Local::now();
      ctx.write(Msg::new(TypesEnum::Heartbeat, vec![])).await.unwrap_or_else(|e| log::warn!("{:?}", e));
    }
}

#[async_trait::async_trait]
impl Handler<Msg> for ServerHandler {
    async fn read(&mut self, ctx: &Context<Msg>, msg: Msg) {
        match msg.msg_type {
            TypesEnum::Auth => self.auth(ctx, msg).await,
            TypesEnum::ProxyOpen => self.proxy_open(ctx, msg).await,
            TypesEnum::ProxyForward => self.proxy_forward(ctx, msg).await,
            TypesEnum::ProxyClose => self.proxy_close(ctx, msg).await,
            TypesEnum::Heartbeat => self.heartbeat(ctx,msg).await,
            _ => {}
        }
    }
    async fn active(&mut self, ctx: &Context<Msg>) {
        log::info!("trying to connect {:?}", ctx.addr());
    }
    async fn close(&mut self, ctx: &Context<Msg>) {
        log::info!("closed {:?} ", ctx.addr());
        let id = self.id.clone();
        self.is_closed = true;
        self.channel_map.write().await.clear();
        if let Some(id) = id {
            for ele in self.forawrd_shutdown.read().await.iter() {
                ele.1
                    .lock()
                    .await
                    .send(())
                    .await
                    .unwrap_or_else(|e| log::warn!("{:?}", e));
            }
            for task in self.listen_task.iter() {
                task.1.lock().await.abort();
            }
            self.listen_task.clear();
            self.forawrd_shutdown.write().await.clear();
            self.manager.close(id).await;
        }

    }
}
