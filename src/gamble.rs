use std::sync::Arc;

use anyhow::Result;
use log::info;
use nipper::Document;

use crate::{
    config::get_config,
    error::Error,
    gpt::{self, GPTProxy},
    handler::Handler,
};

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub name: String,
    pub start_date: String,
}

pub struct Gamble {}

impl Gamble {
    const USER_ID: &'static str = "26485810-436e-4b7f-8eff-6c771d4efdcc";
    const PROMPT: &'static str = "你是一个爬虫助手，把下面文字格式化， 只返回一周内的比赛，直接返回结果，不要添加任何前置回复：";
    const URL: &'static str = "https://www.aceodds.com/zh-cn/足球/英格兰超级联赛.html";
    pub fn new() -> Self {
        // let gpt_proxy = Arc::new(GPTProxy::new(
        //     get_config().model.clone(),
        //     Gamble::USER_ID.to_string(),
        //     get_config().gpt_api.clone(),
        //     get_config().gpt_token.clone(),
        // ));
        Gamble {}
    }
}

#[async_trait::async_trait]
impl Handler for Gamble {
    async fn on_message(&mut self, msg: &str) -> Result<String> {
        if msg != "戒赌" {
            return Err(Error::NotMatchError)?;
        }
        let html = reqwest::get(Gamble::URL)
            .await
            .map_err(|e| Error::HttpError {
                status: e.status().unwrap(),
                url: e.url().unwrap().to_string(),
                response: e.to_string(),
            })?
            .text()
            .await
            .map_err(|e| Error::HttpError {
                status: e.status().unwrap(),
                url: e.url().unwrap().to_string(),
                response: e.to_string(),
            })?;
        let txt = {
            let document = Document::from(&html); // Confined to this scope
            document.select(".table").first().text().to_string()
        };
       
        let content = format!("{}\n{}", Gamble::PROMPT, txt);
        let res = gpt::queryGPT(
            "gpt-4o-mini".to_string(),
            get_config().gpt_token.clone(),
            content
        ).await?;

        Ok(res)
    }
}

#[tokio::test]
async fn test_gamble() {
    let mut gamble = Gamble::new();
    match gamble.on_message("戒赌").await {
        Ok(v) => println!("{}", v),
        Err(e) => println!("{:?}", e),
    }
}
