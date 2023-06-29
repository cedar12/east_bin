use east_core::encoder::Encoder;




pub struct ForwardEncoder{}

impl Encoder<Vec<u8>> for ForwardEncoder{
    fn encode(&mut self,_ctx:&east_core::context::Context<Vec<u8>>,msg:Vec<u8>,byte_buf:&mut east_core::byte_buf::ByteBuf) {
      byte_buf.write_bytes(&msg).unwrap();
    }
}