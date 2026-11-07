use crate::client::Nidhogg;
use crate::error::NidhoggError;
use serde::Serialize;

impl Nidhogg {
    pub async fn signout(&self, signout: Signout) -> Result<(), NidhoggError> {
        self.client
            .post(format!("{}/authserver/signout", self.yggdrasil_url))
            .json(&signout)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
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
