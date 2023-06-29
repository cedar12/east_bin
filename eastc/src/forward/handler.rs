use std::{collections::HashMap, sync::Arc};

use east_core::{
    byte_buf::ByteBuf, context::Context, handler::Handler, message::Msg, types::TypesEnum,
};
use tokio::sync::RwLock;

pub struct ForwardHandler {
    pub ctx: Context<Msg>,
    pub id: u64,
    pub channel: Arc<RwLock<HashMap<u64, Context<Vec<u8>>>>>,
}

impl ForwardHandler {
    pub fn new(
        ctx: Context<Msg>,
        id: u64,
        channel: Arc<RwLock<HashMap<u64, Context<Vec<u8>>>>>,
    ) -> Self {
        Self {
            ctx: ctx,
            id: id,
            channel: channel,
        }
    }
}

#[async_trait::async_trait]
impl Handler<Vec<u8>> for ForwardHandler {
    async fn read(&mut self, _ctx: &Context<Vec<u8>>, msg: Vec<u8>) {
        let mut bf = ByteBuf::new_with_capacity(0);
        bf.write_u64_be(self.id).unwrap_or_else(|err| {
            log::warn!("{}", err);
            0
        });
        bf.write_bytes(&msg).unwrap_or_else(|err| {
            log::warn!("{}", err);
            0
        });
        let m = Msg::new(TypesEnum::ProxyForward, bf.available_bytes().to_vec());
        self.ctx.write(m).await.unwrap_or_else(|err| {
            log::warn!("{}", err);
        });
    }
    async fn active(&mut self, ctx: &Context<Vec<u8>>) {
        log::debug!("proxy active {:?}", ctx.addr());
        self.channel.write().await.insert(self.id, ctx.clone());
        let mut bf = ByteBuf::new_with_capacity(0);
        bf.write_u64_be(self.id).unwrap_or_else(|err| {
            log::warn!("{}", err);
            0
        });
        let msg = Msg::new(TypesEnum::ProxyOpen, bf.available_bytes().to_vec());
        self.ctx.write(msg).await.unwrap_or_else(|err| {
            log::warn!("{}", err);
        });
        log::debug!("open proxy active {:?}", ctx.addr());
    }
    async fn close(&mut self, ctx: &Context<Vec<u8>>) {
        log::debug!("close {:?} {}", ctx.addr(), self.id);
        self.channel.write().await.remove(&self.id);

        let mut bf = ByteBuf::new_with_capacity(0);
        bf.write_u64_be(self.id).unwrap_or_else(|err| {
            log::warn!("{}", err);
            0
        });
        let msg = Msg::new(TypesEnum::ProxyClose, bf.available_bytes().to_vec());
        self.ctx.write(msg).await.unwrap_or_else(|err| {
            log::warn!("{}", err);
        });
    }
}
