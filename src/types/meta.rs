use crate::error::Result;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Meta {
    #[serde(default)]
    pub meta: ServerMeta,

    #[serde(rename = "skinDomains", default)]
    pub skin_domains: Vec<String>,

    #[serde(rename = "signaturePublickey", default)]
    pub signature_public_key: String,
}

impl Meta {
    /// Encodes the re-serialized metadata as base64.
    ///
    /// Use [`MetadataDocument::raw_base64`] when the original document must be
    /// forwarded without dropping unknown fields.
    pub fn get_base64(&self) -> Result<String> {
        Ok(BASE64_STANDARD.encode(serde_json::to_string(&self)?))
    }
}

/// A metadata response together with its original JSON document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MetadataDocument {
    pub meta: Meta,
    pub raw: String,
}

impl MetadataDocument {
    /// Encodes the original metadata document as base64.
    pub fn raw_base64(&self) -> String {
        BASE64_STANDARD.encode(&self.raw)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct ServerMeta {
    #[serde(rename = "serverName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,

    #[serde(rename = "implementationName")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_name: Option<String>,

    #[serde(rename = "implementationVersion")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation_version: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<Links>,

    #[serde(rename = "feature.non_email_login")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_email_login: Option<bool>,

    #[serde(rename = "feature.legacy_skin_api")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_skin_api: Option<bool>,

    #[serde(rename = "feature.no_mojang_namespace")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_mojang_namespace: Option<bool>,

    #[serde(rename = "feature.enable_mojang_anti_features")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_mojang_anti_features: Option<bool>,

    #[serde(rename = "feature.enable_profile_key")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enable_profile_key: Option<bool>,

    #[serde(rename = "feature.username_check")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username_check: Option<bool>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Links {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub homepage: String,

    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub register: String,
}
