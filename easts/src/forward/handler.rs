use std::{sync::Arc, collections::HashMap};

use east_core::{handler::Handler, context::Context, message::Msg, types::TypesEnum};
use tokio::sync::RwLock;

use super::ForwardMsg;


pub struct ForwardHandler{
  pub ctx:Context<Msg>,
  pub id:u64,
  pub port:u16,
  pub channel:Arc<RwLock<HashMap<u64,Context<ForwardMsg>>>>,
}

impl ForwardHandler{
  pub fn new(ctx:Context<Msg>,id:u64,port:u16,channel:Arc<RwLock<HashMap<u64,Context<ForwardMsg>>>>)->Self{
    Self { ctx: ctx, id: id, port: port, channel: channel }
  }
}

#[async_trait::async_trait]
impl Handler<ForwardMsg> for ForwardHandler{
  async fn read(&mut self, _ctx: &Context<ForwardMsg>, msg: ForwardMsg) {
    let mut bytes=msg.buf;
    let mut id_bytes=self.id.to_be_bytes().to_vec();
    id_bytes.append(&mut bytes);
    let msg=Msg::new(TypesEnum::ProxyForward,id_bytes);
    self.ctx.write(msg).await.unwrap_or_else(|e|log::warn!("{:?}",e));

  }
  async fn active(&mut self, ctx: &Context<ForwardMsg>) {
    log::debug!("forwarding active {:?} id->{}", ctx.addr(),self.id);
    self.channel.write().await.insert(self.id,ctx.clone());

  }
  async fn close(&mut self,ctx:&Context<ForwardMsg>){
    log::debug!("forwarding active close {:?}  id->{}", ctx.addr(),self.id);
    self.channel.write().await.remove(&self.id);

  }
}


