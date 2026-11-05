use serde::{Deserialize, Serialize};
use crate::structs::profile::Profile;
use crate::structs::user::User;

#[derive(Serialize, Debug)]
pub struct Authenticate {
    pub username: String,

    pub password: String,

    #[serde(rename = "clientToken")]
    pub client_token: Option<String>,

    #[serde(rename = "requestUser")]
    pub request_user: bool,

    pub agent: Agent
}

impl Authenticate {
    pub fn new(username: String, password: String, client_token: Option<String>, request_user: bool, agent: Agent) -> Authenticate {
        Self {
            username, password, client_token, request_user, agent,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Agent {
    pub name: String,
    pub version: i64
}

impl Agent {
    pub fn default() -> Self {
        Self { name: "Minecraft".to_string(), version: 1 }
    }
}

#[derive(Deserialize, Debug)]
pub struct AuthenticateResponse {
    #[serde(rename = "accessToken")]
    pub access_token: String,

    #[serde(rename = "clientToken")]
    pub client_token: String,

    #[serde(rename = "availableProfiles")]
    pub available_profiles: Option<Vec<Profile>>,

    #[serde(rename = "selectedProfile")]
    pub selected_profile: Option<Profile>,

    pub user: Option<User>
}