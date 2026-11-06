use crate::client::{Nidhogg, NidhoggError};
use crate::structs::profile::Profile;

impl Nidhogg {
    pub async fn get_profile(&self, uuid: String, unsigned: bool) -> Result<Profile, NidhoggError> {
        let url = format!("/sessionserver/session/minecraft/profile/{uuid}?unsigned={unsigned}");
        let response = self.get_request(url.as_str()).await?;

        match response {
            None => Err(NidhoggError::Other("No response from server".to_string())),
            Some(response) => {
                let profile: Profile = serde_json::from_str(&response)?;
                Ok(profile)
            }
        }
    }

    pub async fn get_profiles(&self, names: Vec<String>) -> Result<Vec<Profile>, NidhoggError> {
        let body = serde_json::to_string(&names)?;
        let response = self
            .post_request("/api/profiles/minecraft", body.as_str())
            .await?;

        match response {
            None => Err(NidhoggError::Other("No response from server".to_string())),
            Some(response) => {
                let profiles: Vec<Profile> = serde_json::from_str(&response)?;
                Ok(profiles)
            }
        }
    }
}
