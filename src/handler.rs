use std::sync::Arc;
use log::info;
use tokio::sync::Mutex;

use anyhow::Result;

use crate::error::Error;

#[async_trait::async_trait]
pub trait Handler : Send + Sync{
    async fn on_message(&mut self, msg: &str) -> Result<String>;
}

#[derive(Default)]
pub struct HandlerMgr {
    handlers: Arc<Mutex<Vec<Arc<Mutex<dyn Handler>>>>>,
}

impl HandlerMgr {
    pub fn new() -> Self {
       Self::default()
    }

    pub async fn register_handler(&mut self, handler: Arc<Mutex<dyn Handler>>) {
        let mut handlers = self
            .handlers
            .lock()
            .await;
            // .expect("Failed to acquire handlers lock");
        handlers.push(handler);
    }

    pub async fn match_handler(&self, msg: &str) -> Result<String> {
        let handlers = self
            .handlers
            .lock()
            .await;
        info!("try match msg: {}, handler size: {}", msg, handlers.len());
        for h in handlers.iter() {
            let mut h = h.lock().await;
            match h.on_message(msg).await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    if e.to_string() == Error::NotMatchError.to_string() {
                        continue;
                    }
                    return Err(e);
                },
            }
        }
        Err(Error::NotMatchError)?
    }
}
