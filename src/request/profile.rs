use crate::client::Nidhogg;
use crate::error::Result;
use crate::types::profile::Profile;

impl Nidhogg {
    pub async fn get_profile(&self, uuid: String, unsigned: bool) -> Result<Profile> {
        Ok(self
            .client
            .get(format!(
                "{}/sessionserver/session/minecraft/profile/{uuid}?unsigned={unsigned}",
                self.yggdrasil_url
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<Profile>()
            .await?)
    }

    pub async fn get_profiles(&self, names: Vec<String>) -> Result<Vec<Profile>> {
        Ok(self
            .post_json("/api/profiles/minecraft", &names)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Profile>>()
            .await?)
    }
}
