use crate::client::{Nidhogg, NidhoggError};
use serde::Serialize;

impl Nidhogg {
    pub async fn validate(&self, validate: &Tokens) -> Result<(), NidhoggError> {
        let body = serde_json::to_string(validate)?;
        let response = self.post_request("/authserver/validate", &body).await?;

        match response {
            None => Ok(()),
            Some(response) => Err(NidhoggError::Other(format!(
                "Unexpected response from Yggdrasil: {}",
                response
            ))),
        }
    }

    pub async fn invalidate(&self, validate: &Tokens) -> Result<(), NidhoggError> {
        let body = serde_json::to_string(validate)?;
        let response = self.post_request("/authserver/invalidate", &body).await?;

        match response {
            None => Ok(()),
            Some(response) => Err(NidhoggError::Other(format!(
                "Unexpected response from Yggdrasil: {}",
                response
            ))),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Tokens {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,
}

impl Tokens {
    pub fn new(access_token: String, client_token: Option<String>) -> Self {
        Tokens {
            access_token,
            client_token,
        }
    }
}
