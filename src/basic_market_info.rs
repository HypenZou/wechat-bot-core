use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use log::info;
use rand::{Rng, seq::IndexedRandom};

use crate::{config::get_config, error::Error, proxy::Message, handler::Handler};

static good_prompt_arrary: [&'static str; 4] = [
    "买了。我买回了我卖掉的一切。我拥有的每一枚硬币都回来了。我完全重返了市场，激进的购买、巨大的泵，一切都那么享受。市场起飞了，我入场了。",
    "出10wu",
    "狂暴大牛牛",
    "洗脚去了",
];

static bad_prompt_arrary: [&'static str; 5] = [
    "卖了。我卖掉了我所有的一切，我完全退出了加密货币市场，我再也受不了了。激进的倾销、操纵，巨大的崩，一切都那么激烈。加密结束了，我离开了。",
    "不怕，现货不怕",
    "先套住，再研究",
    "这是价值投资",
    "没关系，技术性回调, 跟他耍耍",
];

static normal_prompt_arrary: [&'static str; 1] = ["沉淀"];

pub struct BasicMakertInfo {}

impl BasicMakertInfo {
    const PROMPTS_1: &'static str = "牛回";
    const PROMPTS_2: &'static str = "牛死";
    pub fn new() -> Self {
        BasicMakertInfo {}
    }
}

#[async_trait::async_trait]
impl Handler for BasicMakertInfo {
    async fn on_message(&mut self, msg: &str) -> Result<String> {
        info!("basic market info msg: {}", msg);
        if msg != BasicMakertInfo::PROMPTS_1 && msg != BasicMakertInfo::PROMPTS_2 {
            return Err(Error::NotMatchError)?;
        }
        get_basic_info().await
    }
}

pub async fn get_basic_info() -> Result<String> {
    let mut result_list = Vec::new();
    let ixic = get_basic_index("IXIC").await?;
    result_list.push(("纳指", ixic.0, ixic.1));
    let hsi = get_basic_index("HSI").await?;
    result_list.push(("恒指", hsi.0, hsi.1));
    let btc = get_crypto_diff("BTC-USDT").await?;
    result_list.push(("BTC", btc.0, btc.1));
    let eth = get_crypto_diff("ETH-USDT").await?;
    result_list.push(("ETH", eth.0, eth.1));
    let sol = get_crypto_diff("SOL-USDT").await?;
    result_list.push(("SOL", sol.0, sol.1));

    let mut cnt = 0;
    for (_, a, b) in &mut result_list {
        if *a > *b {
            cnt += 1;
        }
        *b = (*a - *b) / *b * 100.0
    }

    let mut str = String::from("");
    for (name, cur, diff) in result_list {
        str += &format!("{} {} {:.2}%\n", name, cur, diff);
    }
    if cnt >= 3 {
        let mut rng = rand::thread_rng();
        let v = good_prompt_arrary.choose(&mut rng).unwrap();
        str += v;
    } else if cnt <= 2 {
        let mut rng = rand::thread_rng();
        let v = bad_prompt_arrary.choose(&mut rng).unwrap();
        str += v;
    } else {
        let mut rng = rand::thread_rng();
        let v = normal_prompt_arrary.choose(&mut rng).unwrap();
        str += v;
    }
    Ok(str)
}

async fn get_basic_index(ticker: &str) -> Result<(f64, f64)> {
    let mut inxids = 0;
    if ticker == "IXIC" {
        inxids = 1114;
    }
    if ticker == "HSI" {
        inxids = 1015;
    }
    let url = format!(
        "https://sapi.k780.com/?app=finance.globalindex&inxids={}&appkey={}&sign={}&format=json",
        inxids,
        get_config().nowapi_appkey,
        get_config().nowapi_token,
    );
    let resp = reqwest::get(&url)
        .await
        .map_err(|e| Error::HttpError {
            status: e.status().unwrap(),
            url: e.url().unwrap().to_string(),
            response: e.to_string(),
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| Error::JsonError(e.to_string()))?
        .pointer(&format!("/result/lists/{}", inxids))
        .ok_or(Error::ResultError("failed to get result for index"))?
        .clone();

    let previous = resp
        .pointer("/yesy_price")
        .ok_or(Error::ResultError("no previous info"))?
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let cur = resp
        .pointer("/last_price")
        .ok_or(Error::ResultError("no cur info"))?
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    Ok((cur, previous))
}

async fn get_crypto_price(ticker: &str, time: DateTime<Utc>) -> Result<f64> {
    let url = format!(
        "https://www.okx.com/api/v5/market/history-candles?instId={}&after={}",
        ticker,
        time.timestamp_millis()
    );
    let res = reqwest::get(&url)
        .await
        .map_err(|e| Error::HttpError {
            status: e.status().unwrap(),
            url: e.url().unwrap().to_string(),
            response: e.to_string(),
        })?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| Error::JsonError(e.to_string()))?
        .pointer("/data")
        .ok_or(Error::ResultError("failed to get data"))?
        .as_array()
        .ok_or(Error::JsonError("failed to parse data".to_string()))?
        .get(0)
        .ok_or(Error::ResultError("no data"))?
        .as_array()
        .unwrap()
        .get(1)
        .unwrap()
        .as_str()
        .unwrap()
        .parse::<f64>()
        .unwrap();

    Ok(res)
}

async fn get_crypto_diff(ticker: &str) -> Result<(f64, f64)> {
    let now = get_crypto_price(ticker, Utc::now()).await?;
    let day_ago = get_crypto_price(ticker, Utc::now() - Duration::days(1)).await?;
    Ok((now, day_ago))
}

#[tokio::test]
async fn test_get_basic_index() {
    match get_basic_index("IXIC").await {
        Ok(v) => println!("{:?}", v),
        Err(e) => println!("{:?}", e),
    }
}
#[tokio::test]
async fn test_get_crypto_price() {
    match get_crypto_price("ETH-USDT", Utc::now()).await {
        Ok(v) => println!("{:?}", v),
        Err(e) => println!("{:?}", e),
    }
}
