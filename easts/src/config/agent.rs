use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Agent{
   pub bind_port:u16,
   #[serde(default = "default_host")]
   pub target_host:String,
   pub target_port:u16,
   #[serde(default = "defualt_max_rate")]
   pub max_rate:Option<u32>,
   #[serde(default = "default_whitelist")]
   pub whitelist:Vec<String>
}

fn default_host() -> String{
    String::from("127.0.0.1")
}

fn default_whitelist()->Vec<String>{
    vec![]
}

fn defualt_max_rate()->Option<u32>{
    None
}
