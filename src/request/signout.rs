use crate::client::{Nidhogg, NidhoggError};
use serde::Serialize;

impl Nidhogg {
    pub async fn signout(&self, signout: Signout) -> Result<(), NidhoggError> {
        let body = serde_json::to_string(&signout)?;
        let response = self.post_request("/authserver/signout", &body).await?;

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
pub struct Signout {
    pub username: String,
    pub password: String,
}

impl Signout {
    pub fn new(username: String, password: String) -> Self {
        Signout { username, password }
    }
}
