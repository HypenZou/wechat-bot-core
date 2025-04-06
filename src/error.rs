use reqwest::StatusCode;
use thiserror::Error;
use anyhow::{Context, Result};

#[derive(Debug, Error)]
pub enum Error {

    #[error("Invalid parameters: {0}")]
    ParamError(String),

    #[error("HTTP error {status}: {url}")]
    HttpError {
        status: StatusCode,
        url: String,
        response: String,
    },

    #[error("JSON parse error: {0}")]
    JsonError(String),

    #[error("get error: {0}")]
    ResultError(&'static str),

    #[error("not match")]
    NotMatchError,
}