use crate::config;


#[derive(Debug,Clone)]
pub struct Agent{
  pub id:String,
  pub name:String,
  pub forward:Vec<Forward>,
}

#[derive(Debug,Clone)]
pub struct Forward{
  pub bind_port:u16,
  pub target_host:String,
  pub target_port:u16,
  pub enable:bool,
  pub whitelist:Vec<String>,
  pub max_rate:Option<u32>,
}


fn get_conf_agent(id:String)->Option<Agent>{
  match config::CONF.agent.get(&id){
    Some(agents)=>{
      return Some(
        Agent{
          id:id.clone(),
          name: id.clone(),
          forward: agents.iter().map(move |a|Forward{
            bind_port: a.bind_port,
            target_host: a.target_host.clone(),
            target_port: a.target_port,
            enable: true,
            whitelist: a.whitelist.clone(),
            max_rate: a.max_rate,
          }).collect(),
        })
    },
    None=>{
      return None
    }
  }
}

pub fn get_agent(id:String)->Option<Agent>{


  get_conf_agent(id)
}