use std::sync::Arc;
use tokio::sync::Mutex;

use crate::proxy::{
    Message, MessageResp,
    proxy_server::{Proxy, ProxyServer},
};
use tonic::{Request, Response, Status, transport::Server};
use wechat_bot_core::{
    basic_market_info::BasicMakertInfo, config::get_config, gamble::Gamble, gpt::GPTProxy,
    handler::HandlerMgr, huangli::HuangLi, proxy::RespCode, *,
};
// Import the generated proto-rust file into a module

use log::{error, info};

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
                    error!("failed to excute instruction, err: {:?}", e);
                    let resp = MessageResp {
                        code: RespCode::Ok.into(),
                        response: String::from("哦豁"),
                    };
                    return Ok(Response::new(resp));
                }
            }
        }
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
// Runtime to run our server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let mut proxy = ProxyService::default();
    proxy.register_handlers().await;
    println!("Starting gRPC Server...");
    env_logger::init();

    Server::builder()
        .add_service(ProxyServer::new(proxy))
        .serve(addr)
        .await?;
    Ok(())
}
