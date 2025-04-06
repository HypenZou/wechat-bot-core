use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::{Context, Result};
use reqwest::StatusCode;
use serde_json::json;
use tonic::async_trait;

use crate::{config::get_config, error::Error};

pub struct GPTProxy {
    model: String,
    user_id: String,
    client: Arc<Mutex<reqwest::Client>>,
    url: String,
    token: String,
    pre_set: String,
}

pub async fn queryGPT(model: String, token: String, content: String) -> Result<String> {
    let body = json!({
     "model": model,
     "messages" :[{
         "role": "user",
         "content":content
     }]
    });
    let url = "https://api.302.ai/v1/chat/completions";
    let body_str = serde_json::to_string(&body).unwrap();
    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("Authorization", get_config().gpt_token.clone())
        .body(body_str)
        .send()
        .await
        .map_err(|e| Error::HttpError {
            status: e.status().unwrap(),
            url: e.url().unwrap().to_string(),
            response: e.to_string(),
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| Error::JsonError(e.to_string()))?;
    println!("{}", serde_json::to_string(&resp).unwrap());

    let res: Vec<String> = resp
        .pointer("/choices")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            return v
                .pointer("/message/content")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
        })
        .collect();
    if res.is_empty() {
        return Err(Error::HttpError {
            status: StatusCode::SEE_OTHER,
            url: url.to_string(),
            response: String::from("content empty"),
        })?;
    }
    Ok(res.get(0).unwrap().to_string())
}

impl GPTProxy {
    const CHAT_API: &'static str = "/chat/completions";
    pub fn new(
        model: String,
        user_id: String,
        url: String,
        token: String,
        pre_set: String,
    ) -> Self {
        let client = Arc::new(Mutex::new(reqwest::Client::new()));
        GPTProxy {
            model,
            user_id,
            client,
            url: format!("{}{}", url, GPTProxy::CHAT_API).to_string(),
            token,
            pre_set,
        }
    }

    pub async fn test(&mut self) -> Arc<String> {
        println!("test");
        Arc::new(String::from("test"))
    }

    pub async fn query(&mut self, content: String) -> Result<String> {
        let final_content = format!("{}{}", self.pre_set, content);
        let body = json!({
         "model": self.model.clone(),
         "messages" :[{
             "role": "user",
             "content":final_content
         }],
         "userid": self.user_id
        });
        let body_str = serde_json::to_string(&body).unwrap();
        let resp = self
            .client
            .lock()
            .await
            .post(&self.url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", &self.token)
            .body(body_str)
            .send()
            .await
            .map_err(|e| Error::HttpError {
                status: e.status().unwrap(),
                url: e.url().unwrap().to_string(),
                response: e.to_string(),
            })?
            .json::<serde_json::Value>()
            .await
            .map_err(|e| Error::JsonError(e.to_string()))?;
        info!("{}", serde_json::to_string(&resp).unwrap());

        let res: Vec<String> = resp
            .pointer("/choices")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .map(|v| {
                return v
                    .pointer("/message/content")
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_string();
            })
            .collect();
        if res.is_empty() {
            return Err(Error::HttpError {
                status: StatusCode::SEE_OTHER,
                url: self.url.clone(),
                response: String::from("content empty"),
            })?;
        }
        let mut res = res.get(0).unwrap().to_string();
        res = res.trim_start().to_string();
        Ok(res)
    }
}

#[tokio::test]
async fn test_gpt() {
    let mut gpt_proxy = GPTProxy::new(
        get_config().model.clone(),
        get_config().user_id.clone(),
        get_config().gpt_api.clone(),
        get_config().gpt_token.clone(),
        String::from(""),
    );
    match gpt_proxy.query(String::from("你好")).await {
        Ok(v) => println!("{}", v),
        Err(e) => println!("{:?}", e),
    }
}
