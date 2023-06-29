use std::time::Duration;

use chrono::{DateTime, Local};
use east_core::{decoder::Decoder, byte_buf::ByteBuf};
use tokio::time;

use super::ForwardMsg;


pub struct ForwardDecoder{
  start:DateTime<Local>,
  n:u32,
  max:Option<u32>,
}

impl ForwardDecoder {
    pub fn new(max:Option<u32>)->Self{
      Self { start: Local::now() ,n:0,max:max}
    }
}


#[async_trait::async_trait]
impl Decoder<ForwardMsg> for ForwardDecoder{
    async fn decode(&mut self,ctx: &east_core::context::Context<ForwardMsg> ,byte_buf: &mut ByteBuf) {
      let len=byte_buf.readable_bytes();
      if len==0{
        return
      }
      if let Some(max)=self.max{
        let max=max*1024;
        let duration=Local::now()-self.start;
        if duration.num_seconds()>=1{
          if self.n>max{
            let secs=(self.n as f64)/(max as f64)-1f64;
            log::debug!("waiting {}s",secs);
            time::sleep(Duration::from_secs_f64(secs)).await;
          }
          self.n=0;
          self.start=Local::now();
        }else if self.n>max{
          let secs=(self.n as f64)/(max as f64)-(duration.num_milliseconds() as f64/1000f64);
          log::debug!("waiting {}s",secs);
          time::sleep(Duration::from_secs_f64(secs)).await;
          self.n=0;
          self.start=Local::now();
        }
        self.n+=len as u32;
      }
      let mut buf=vec![0u8;len];
      byte_buf.read_bytes(&mut buf);
      byte_buf.clean();
      
      ctx.out(ForwardMsg{buf:buf}).await.unwrap_or_else(|e|log::warn!("{:?}",e));
      
    }
    
}