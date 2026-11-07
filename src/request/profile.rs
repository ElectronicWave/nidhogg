use crate::client::Nidhogg;
use crate::error::NidhoggError;
use crate::structs::profile::Profile;

impl Nidhogg {
    pub async fn get_profile(&self, uuid: String, unsigned: bool) -> Result<Profile, NidhoggError> {
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

    pub async fn get_profiles(&self, names: Vec<String>) -> Result<Vec<Profile>, NidhoggError> {
        Ok(self
            .client
            .post(format!("{}/api/profiles/minecraft", self.yggdrasil_url))
            .json(&names)
            .send()
            .await?
            .error_for_status()?
            .json::<Vec<Profile>>()
            .await?)
    }
}
