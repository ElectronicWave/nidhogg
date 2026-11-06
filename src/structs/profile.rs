use crate::structs::textures::TextureProperty;
use crate::structs::Property;
use base64::prelude::BASE64_STANDARD;
use base64::{DecodeError, Engine};
use serde::{Deserialize, Serialize};
use std::string::FromUtf8Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProfilePropertyError {
    #[error(transparent)]
    DecodeBase64(#[from] DecodeError),

    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),

    #[error(transparent)]
    Deserialize(#[from] serde_json::Error),

    #[error("No `{0}` property found in profile")]
    NoPropertyFound(String),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Profile {
    /// Profile UUID (Unhyphenated)
    pub id: String,

    /// Profile name
    pub name: String,

    /// Profile properties (only needed in specific cases)
    pub properties: Option<Vec<Property>>,
}

impl Profile {
    fn get_property(&self, name: &str) -> Option<&Property> {
        self.properties.as_ref()?.iter().find(|p| p.name == name)
    }

    pub fn get_textures(&self) -> Result<TextureProperty, ProfilePropertyError> {
        match self.get_property("textures") {
            None => Err(ProfilePropertyError::NoPropertyFound(
                "textures".to_string(),
            )),
            Some(p) => {
                let json = String::from_utf8(BASE64_STANDARD.decode(&p.value)?)?;
                Ok(serde_json::from_str(&*json)?)
            }
        }
    }

    pub fn get_uploadable_textures(&self) -> Result<Vec<String>, ProfilePropertyError> {
        match self.get_property("uploadableTextures") {
            None => Err(ProfilePropertyError::NoPropertyFound(
                "uploadableTextures".to_string(),
            )),
            Some(p) => {
                let mut uploadable_textures: Vec<String> = Vec::new();
                for text in p.value.split(",") {
                    uploadable_textures.push(text.to_string());
                }
                Ok(uploadable_textures)
            }
        }
    }
}
