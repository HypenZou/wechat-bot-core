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
    async fn on_message(&mut self, msg: &str) -> Result<String> {
        info!("help msg: {}", msg);
        if msg != Help::PROMPT {
            return Err(Error::NotMatchError)?;
        }
        
        let help_text = r#"可用指令列表：
/help - 显示此帮助信息
/牛回 或 /牛死 - 炒股biss
/算命 - 迷信biss
/戒赌 - 赌狗biss

其他问题直接@你爹"#;
        
        Ok(help_text.to_string())
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