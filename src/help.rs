use anyhow::Result;
use log::info;

use crate::error::Error;
use crate::handler::Handler;

pub struct Help {}

impl Help {
    const PROMPT: &'static str = "help";
    
    pub fn new() -> Self {
        Help {}
    }
}

#[async_trait::async_trait]
impl Handler for Help {
    async fn on_message(&mut self, _msg: &str) -> Result<String> {
        // 空函数体
        Ok(String::new())
    }
    
    fn help(&self) -> String {
        "help - 显示此帮助信息".to_string()
    }
}

#[tokio::test]
async fn test_help() {
    let mut help = Help::new();
    match help.on_message("help").await {
        Ok(v) => println!("{}", v),
        Err(e) => println!("{:?}", e),
    }
} 