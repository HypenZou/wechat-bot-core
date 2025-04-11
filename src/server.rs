use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use std::{fs::File, sync::Arc};
use tokio::sync::Mutex;

use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use crate::proxy::{
    Message, MessageResp,
    proxy_server::{Proxy, ProxyServer},
};
use tonic::{Request, Response, Status, transport::Server};
use wechat_bot_core::{
    basic_market_info::BasicMakertInfo, config::get_config, gamble::Gamble, gpt::GPTProxy,
    handler::HandlerMgr, huangli::HuangLi, proxy::RespCode, *,
    help::Help,
};
// Import the generated proto-rust file into a module

use log::{LevelFilter, error, info};

#[derive(Default)]
pub struct ProxyService {
    handlers: Arc<Mutex<HandlerMgr>>,
}
impl ProxyService {
    pub async fn register_handlers(&mut self) {
        let mut handlers = self.handlers.lock().await;
        handlers
            .register_handler(Arc::new(Mutex::new(BasicMakertInfo::new())))
            .await;
        handlers
            .register_handler(Arc::new(Mutex::new(Gamble::new())))
            .await;
        handlers
            .register_handler(Arc::new(Mutex::new(HuangLi::new())))
            .await;
        handlers
            .register_handler(Arc::new(Mutex::new(Help::new())))
            .await;
    }

    async fn handle_help(&self) -> String {
        let handlers = self.handlers.lock().await;
        let mut help_text = String::from("可用指令列表：\n");
        
        for handler in handlers.iter() {
            help_text.push_str(&format!("{}\n", handler.lock().await.help()));
        }
        
        help_text.push_str("其他问题直接@你爹");
        help_text
    }
}
#[tonic::async_trait]
impl Proxy for ProxyService {
    async fn on_message(
        &self,
        req: tonic::Request<Message>,
    ) -> std::result::Result<tonic::Response<MessageResp>, tonic::Status> {
        // filter
        let msg = req.get_ref();
        let is_target = msg.is_room
            && (msg.room_id == get_config().room_id_dev || msg.room_id == get_config().room_id)
            && msg.is_memtioned;
        if !is_target {
            let resp = MessageResp {
                code: RespCode::Ignore.into(),
                response: String::from(""),
            };
            return Ok(Response::new(resp));
        }
        let content = msg.content.trim();
        // help function
        if content == "/help" {
            let help_text = self.handle_help().await;
            return Ok(Response::new(MessageResp {
                code: RespCode::Ok.into(),
                response: help_text,
            }));
        }
        // Follow the instruction
        if content.starts_with("/") {
            let hs = self.handlers.lock().await;
            match hs
                .match_handler(&content.chars().skip(1).collect::<String>())
                .await
            {
                Ok(v) => {
                    let resp = MessageResp {
                        code: RespCode::Ok.into(),
                        response: v,
                    };
                    return Ok(Response::new(resp));
                }
                Err(e) => {
                    error!("failed to execute instruction, err: {:?}", e);
                    let resp = MessageResp {
                        code: RespCode::Ok.into(),
                        response: String::from("哦豁"),
                    };
                    return Ok(Response::new(resp));
                }
            }
        }
        // Respond to the message
        let mut gpt_proxy = GPTProxy::new(
            get_config().model.clone(),
            get_config().user_id.clone(),
            get_config().gpt_api.clone(),
            get_config().gpt_token.clone(),
            get_config().tieba_pre_set.clone(),
        );

        match gpt_proxy.query(content.to_string()).await {
            Ok(v) => {
                let resp = MessageResp {
                    code: RespCode::Ok.into(),
                    response: v,
                };
                return Ok(Response::new(resp));
            }
            Err(e) => {
                let resp = MessageResp {
                    code: RespCode::Ok.into(),
                    response: String::from("哦豁"),
                };
                return Ok(Response::new(resp));
            }
        }
    }
}

fn init_logger() {
    let log_roller = FixedWindowRoller::builder()
        .build("logs/archive/app_{}.log", 5) // 保留5个历史文件
        .unwrap();
    let log_trigger = SizeTrigger::new(1024 * 1024); // 1MB触发滚动
    let log_policy = CompoundPolicy::new(Box::new(log_trigger), Box::new(log_roller));

    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} - {m}{n}")))
        .build("logs/app.log", Box::new(log_policy))
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();
    // 初始化日志系统
    log4rs::init_config(config).unwrap();
}

// Runtime to run our server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let mut proxy = ProxyService::default();
    proxy.register_handlers().await;
    init_logger();
    Server::builder()
        .add_service(ProxyServer::new(proxy))
        .serve(addr)
        .await?;
    info!("Starting gRPC Server...");
    Ok(())
}
