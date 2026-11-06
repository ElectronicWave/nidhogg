use crate::client::{Nidhogg, NidhoggError};
use crate::structs::profile::Profile;
use crate::structs::user::User;
use serde::{Deserialize, Serialize};

impl Nidhogg {
    pub async fn refresh(&self, refresh: &Refresh) -> Result<RefreshResponse, NidhoggError> {
        let body = serde_json::to_string(refresh)?;
        let response = self.post_request("/authserver/refresh", &body).await?;

        match response {
            Some(response) => {
                let refresh_response: RefreshResponse = serde_json::from_str(&response)?;
                Ok(refresh_response)
            }
            None => Err(NidhoggError::Other("No response from server".to_string())),
        }
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
