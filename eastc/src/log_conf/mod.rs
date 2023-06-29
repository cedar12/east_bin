use chrono::{Local, DateTime};
use cron::Schedule;
use log::{LevelFilter};
use std::str::FromStr;
use log4rs::{Config, append::{console::ConsoleAppender, file::FileAppender}, encode::pattern::PatternEncoder, config::{Appender, Root, Logger}, Handle};
use tokio::spawn;

use crate::config;

const PATTERN_ENCODER:&str="[EAST] {d(%Y-%m-%d %H:%M:%S)} - {l} - {t}:{L} - {m}{n}";
const DATETIME_FORMAT:&str="%Y%m%d";
const CRON:&str="0 0 0 * * * *";

fn get_log_config()->Config{
    let now=Local::now().format(DATETIME_FORMAT);
  let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(PATTERN_ENCODER)))
        .build();

    let file = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(PATTERN_ENCODER)))
        .build(format!("log/{}.log",now.to_string()))
        .unwrap();
   
    let level=from_env_level();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(Logger::builder()
            .appender("file")
            .additive(true)
            
            .build("EAST", level))
        .build(Root::builder().appender("stdout").appender("file").build(level))
        .unwrap();
    config
}


fn from_env_level()->LevelFilter{
    let log_level=config::CONF.log_level.clone();
    match log_level{
        Some(level)=>{
            match level.as_str() {
                "off"=>{
                    LevelFilter::Off
                },
                "error"=>{
                    LevelFilter::Error
                },
                "warn"=>{
                    LevelFilter::Warn
                },
                "info"=>{
                    LevelFilter::Info
                },
                "trace"=>{
                    LevelFilter::Trace
                },
                "debug"=>{
                    LevelFilter::Debug
                },
                _=>{
                    LevelFilter::Info
                }
            }
        }
        None=>LevelFilter::Info
    }
    
}


fn init_config()->Handle{
    let config=get_log_config();
    log4rs::init_config(config).unwrap()
}


pub fn init(){
    let mut handle=init_config();
    spawn(async move{
        schedule_task(&mut handle);
    });
}

fn schedule_task(handle:&mut Handle) {
    // let schedule = Schedule::from_str("0/10 * * * * * *").unwrap();
    let schedule = Schedule::from_str(CRON).unwrap();

    loop {
        let now: DateTime<Local> = Local::now();
        let next = schedule.upcoming(Local).next().unwrap();

        let delay = next - now;
        let delay_millis=delay.num_milliseconds() as u64;
        std::thread::sleep(std::time::Duration::from_millis(delay_millis));
        if delay_millis>100{
            handle.set_config(get_log_config());
        }
    }
}