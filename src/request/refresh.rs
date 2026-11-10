use crate::client::Nidhogg;
use crate::error::NidhoggError;
use crate::types::profile::Profile;
use crate::types::user::User;
use serde::{Deserialize, Serialize};

impl Nidhogg {
    pub async fn refresh(&self, refresh: &Refresh) -> Result<RefreshResponse, NidhoggError> {
        Ok(self
            .client
            .post(format!("{}/authserver/refresh", self.yggdrasil_url))
            .json(refresh)
            .send()
            .await?
            .error_for_status()?
            .json::<RefreshResponse>()
            .await?)
    }
}

#[derive(Serialize, Debug)]
pub struct Refresh {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,

    #[serde(rename = "requestUser")]
    pub request_user: bool,

    #[serde(rename = "selectedProfile")]
    pub selected_profile: Option<Profile>,
}

impl Refresh {
    pub fn new(
        access_token: String,
        client_token: Option<String>,
        request_user: bool,
        selected_profile: Option<Profile>,
    ) -> Self {
        Refresh {
            access_token,
            client_token,
            request_user,
            selected_profile,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RefreshResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "clientToken")]
    pub client_token: String,

    #[serde(rename = "selectedProfile")]
    pub selected_profile: Option<Profile>,

    pub user: Option<User>,
}
