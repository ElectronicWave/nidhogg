use crate::error::NidhoggError;
use crate::types::meta::Meta;
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};

pub struct Nidhogg {
    pub yggdrasil_url: String,
    pub(crate) client: Client,
}

impl Nidhogg {
    pub fn new(yggdrasil_url: &str) -> Result<Self, NidhoggError> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));
        headers.insert("Accept-Charset", HeaderValue::from_static("UTF-8"));

        let client = Client::builder()
            .user_agent("nidhogg/0.1.0")
            .default_headers(headers)
            .build()?;

        Ok(Nidhogg {
            yggdrasil_url: yggdrasil_url.to_string(),
            client,
        })
    }

    pub async fn metadata(&self) -> Result<Meta, NidhoggError> {
        let response = self
            .client
            .get(format!("{}/", self.yggdrasil_url))
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json().await?)
    }
}
