use anyhow::Result;
use chrono::Local;
use log::info;

use crate::{config::get_config, error::Error, handler::Handler};

pub struct HuangLi {
    api_key: String,
}

impl HuangLi {
    const prompt: &'static str = "算命";
    const url: &'static str = "http://v.juhe.cn/laohuangli/d";
    pub fn new() -> Self {
        HuangLi {
            api_key: get_config().huangli_apikey.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Handler for HuangLi {
    async fn on_message(&mut self, msg: &str) -> Result<String> {
        info!("gamble msg: {}", msg);
        if msg != HuangLi::prompt {
            Err(Error::NotMatchError)?
        }
        let now = Local::now().format("%Y-%m-%d").to_string();
        let url = format!("{}?date={}&key={}", HuangLi::url, &now, &self.api_key);
        let resp = reqwest::get(&url)
            .await
            .map_err(|e| Error::HttpError {
                status: e.status().unwrap(),
                url: e.url().unwrap().to_string(),
                response: e.to_string(),
            })?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| Error::JsonError(String::from("failed to parse to json value")))?
            .pointer("/result")
            .ok_or(Error::ResultError("no result"))?
            .clone();

        let yi = resp
            .pointer("/yi")
            .ok_or(Error::ResultError("invalid result for yi"))?
            .as_str()
            .ok_or(Error::ResultError("parse failed for yi"))?;
        let ji = resp
            .pointer("/ji")
            .ok_or(Error::ResultError("invalid result for ji"))?
            .as_str()
            .ok_or(Error::ResultError("parse failed for ji"))?;

        Ok(format!("宜：{}\n忌: {}", yi, ji))
    }
}

#[tokio::test]
async fn test_huangli() {
    let mut huangli = HuangLi::new();
    match huangli.on_message("算命").await {
        Ok(v) => println!("{}", v),
        Err(e) => println!("{:?}", e),
    }
}
