use crate::client::Nidhogg;
use crate::error::NidhoggError;
use serde::Serialize;

impl Nidhogg {
    pub async fn validate(&self, validate: &Tokens) -> Result<(), NidhoggError> {
        self.client
            .post(format!("{}/authserver/validate", self.yggdrasil_url))
            .json(validate)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn invalidate(&self, validate: &Tokens) -> Result<(), NidhoggError> {
        self.client
            .post(format!("{}/authserver/invalidate", self.yggdrasil_url))
            .json(validate)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
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
