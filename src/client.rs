use std::str::Utf8Error;
use reqwest::{Client, Method, RequestBuilder};
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use thiserror::Error;
use crate::client::NidhoggError::{ApiError, Other};
use crate::error::Error;

#[derive(Debug, Error)]
pub enum NidhoggError {
    #[error(transparent)]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error(transparent)]
    Utf8(#[from] Utf8Error),

    #[error(transparent)]
    ApiError(#[from] Error),

    #[error("{0}")]
    Other(String),
}

pub struct Nidhogg {
    pub yggdrasil_url: String,
    client: Client
}

impl Nidhogg {
    pub fn new(yggdrasil_url: &str) -> Result<Self, NidhoggError> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_str("application/json;charset=utf-8")?);
        headers.insert("Accept-Charset", HeaderValue::from_str("UTF-8")?);

        let client = Client::builder()
            .user_agent("nidhogg/0.1.0")
            .default_headers(headers)
            .build()?;

        Ok(Nidhogg { yggdrasil_url: yggdrasil_url.to_string(), client })
    }

    pub async fn post_request(&self, url: &str, body: &str) -> Result<Option<String>, NidhoggError> {
        let response = self.client.request(Method::POST, self.yggdrasil_url.clone() + url)
            .body(body.to_string())
            .send().await?;

        let status = response.status();

        if status == reqwest::StatusCode::NO_CONTENT {
            return Ok(None);
        }

        let body = response.bytes().await?;
        let mut response = str::from_utf8(&body)?.to_string();

        while response.starts_with("\u{FEFF}") {
            response = response[3..].to_string();
        }

        if status != reqwest::StatusCode::OK {
            let error: Result<Error, serde_json::Error> = serde_json::from_str(&response);
            return match error {
                Ok(error) => {
                    Err(ApiError(error))
                }
                Err(_error) => {
                    Err(Other(format!("server status: {}, response: {}", status, response)))
                }
            }
        }

        Ok(Some(response))
    }
}