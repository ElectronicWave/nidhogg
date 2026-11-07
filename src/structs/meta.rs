use crate::error::NidhoggError;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize)]
pub struct Meta {
    pub meta: ServerMeta,

    #[serde(rename = "skinDomains")]
    pub skin_domains: Vec<String>,

    #[serde(rename = "signaturePublickey")]
    pub signature_public_key: String,
}

impl Meta {
    pub fn get_base64(&self) -> Result<String, NidhoggError> {
        Ok(BASE64_STANDARD.encode(serde_json::to_string(&self)?))
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ServerMeta {
    #[serde(rename = "serverName")]
    pub server_name: Option<String>,

    #[serde(rename = "implementationName")]
    pub implementation_name: Option<String>,

    #[serde(rename = "implementationVersion")]
    pub implementation_version: Option<String>,

    pub links: Option<Links>,

    #[serde(rename = "feature.non_email_login")]
    pub non_email_login: Option<bool>,

    #[serde(rename = "feature.legacy_skin_api")]
    pub legacy_skin_api: Option<bool>,

    #[serde(rename = "feature.no_mojang_namespace")]
    pub no_mojang_namespace: Option<bool>,

    #[serde(rename = "feature.enable_mojang_anti_features")]
    pub enable_mojang_anti_features: Option<bool>,

    #[serde(rename = "feature.enable_profile_key")]
    pub enable_profile_key: Option<bool>,

    #[serde(rename = "feature.username_check")]
    pub username_check: Option<bool>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct Links {
    pub homepage: String,
    pub register: String,
}
