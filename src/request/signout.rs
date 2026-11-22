use crate::client::Nidhogg;
use crate::error::Result;
use serde::Serialize;

impl Nidhogg {
    pub async fn signout(&self, signout: Signout) -> Result<()> {
        self.post_json("/authserver/signout", &signout)
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
