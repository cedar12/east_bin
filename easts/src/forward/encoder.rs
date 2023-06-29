use std::{thread::sleep, time::Duration};

use chrono::{DateTime, Local};
use east_core::encoder::Encoder;
use super::ForwardMsg;

pub struct ForwardEncoder{
  start:DateTime<Local>,
  n:u32,
  max:Option<u32>,
}

impl ForwardEncoder{
  pub fn new(max:Option<u32>)->Self{
    Self { start: Local::now() ,n:0,max:max}
  }
}


impl Encoder<ForwardMsg> for ForwardEncoder{
    fn encode(&mut self,_ctx:&east_core::context::Context<ForwardMsg>,msg:ForwardMsg,byte_buf:&mut east_core::byte_buf::ByteBuf) {
      if let Some(max)=self.max{
        let len=msg.buf.len();
        let max=max*1024;
        let duration=Local::now()-self.start;
        if duration.num_seconds()>=1{
          if self.n>max{
            let secs=(self.n as f64)/(max as f64)-1f64;
            log::debug!("waiting {}s",secs);
            sleep(Duration::from_secs_f64(secs));
          }
          self.n=0;
          self.start=Local::now();
        }else if self.n>max{
          let secs=(self.n as f64)/(max as f64)-(duration.num_milliseconds() as f64/1000f64);
          log::debug!("waiting {}s",secs);
          sleep(Duration::from_secs_f64(secs));
          self.n=0;
          self.start=Local::now();
        }
        self.n+=len as u32;
      }
      byte_buf.write_bytes(&msg.buf).unwrap_or_else(|e|{log::warn!("{:?}",e);0});
    }
}