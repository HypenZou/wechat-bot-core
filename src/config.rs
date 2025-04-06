use serde::Deserialize;
use std::sync::OnceLock;
use std::{fs, process};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub room_id : String,
    pub room_id_dev: String,

    pub gpt_api: String,
    pub gpt_token: String,
    pub model: String,
    pub user_id: String,

    pub tieba_pre_set: String,

    pub nowapi_token: String,
    pub nowapi_appkey: String,
    pub huangli_apikey: String,
}

// 全局单例实例
static CONFIG: OnceLock<Config> = OnceLock::new();

/// 获取配置单例（首次调用时初始化）
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(|| {
        let config_str = fs::read_to_string("config.json").unwrap_or_else(|e| {
            eprintln!("Failed to read config.json: {e}");
            process::exit(1);
        });

        serde_json::from_str(&config_str).unwrap_or_else(|e| {
            eprintln!("Invalid config format: {e}");
            process::exit(1);
        })
    })
}
