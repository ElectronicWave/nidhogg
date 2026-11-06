use crate::client::NidhoggError::{ApiError, Other};
use crate::error;
use crate::error::Error;
use crate::structs::meta::ServerMeta;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};
use reqwest::multipart::Form;
use reqwest::{Client, Method};
use std::str::Utf8Error;
use thiserror::Error;

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

    #[error(transparent)]
    JsonParse(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}

pub struct Nidhogg {
    pub yggdrasil_url: String,
    client: Client,
}

impl Nidhogg {
    pub fn new(yggdrasil_url: &str) -> Result<Self, NidhoggError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/json;charset=utf-8")?,
        );
        headers.insert("Accept-Charset", HeaderValue::from_str("UTF-8")?);

        let client = Client::builder()
            .user_agent("nidhogg/0.1.0")
            .default_headers(headers)
            .build()?;

        Ok(Nidhogg {
            yggdrasil_url: yggdrasil_url.to_string(),
            client,
        })
    }

    pub async fn metadata(&self) -> Result<ServerMeta, NidhoggError> {
        let response = self.client.request(Method::GET, "/").send().await?;

        if response.status().is_success() {
            let server_meta: ServerMeta = response.json().await?;
            Ok(server_meta)
        } else {
            let error: Error = response.json().await?;
            Err(ApiError(error))
        }
    }

    pub async fn post_request(
        &self,
        url: &str,
        body: &str,
    ) -> Result<Option<String>, NidhoggError> {
        let response = self
            .client
            .request(Method::POST, self.yggdrasil_url.clone() + url)
            .body(body.to_string())
            .send()
            .await?;

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
            return Err(error::error(response));
        }

        Ok(Some(response))
    }

    pub async fn get_request(&self, url: &str) -> Result<Option<String>, NidhoggError> {
        let response = self
            .client
            .request(Method::GET, self.yggdrasil_url.clone() + url)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::NO_CONTENT {
            return Err(Other("204 No Content".to_string()));
        }

        let body = response.bytes().await?;
        let mut response = str::from_utf8(&body)?.to_string();

        while response.starts_with("\u{FEFF}") {
            response = response[3..].to_string();
        }

        if status != reqwest::StatusCode::OK {
            return Err(error::error(response));
        }

        Ok(Some(response))
    }

    pub async fn put_request(
        &self,
        url: &str,
        form: Form,
        access_token: String,
    ) -> Result<Option<String>, NidhoggError> {
        let response = self
            .client
            .request(Method::PUT, self.yggdrasil_url.clone() + url)
            .bearer_auth(access_token)
            .multipart(form)
            .send()
            .await?;

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
            return Err(error::error(response));
        }

        Ok(Some(response))
    }

    pub async fn delete_request(
        &self,
        url: &str,
        access_token: String,
    ) -> Result<(), NidhoggError> {
        let response = self
            .client
            .request(Method::DELETE, self.yggdrasil_url.clone() + url)
            .bearer_auth(access_token)
            .send()
            .await?;

        let status = response.status();

        if status == reqwest::StatusCode::NO_CONTENT {
            Ok(())
        } else {
            let body = response.bytes().await?;
            let mut response = str::from_utf8(&body)?.to_string();

            while response.starts_with("\u{FEFF}") {
                response = response[3..].to_string();
            }

            Err(error::error(response))
        }
    }
}
