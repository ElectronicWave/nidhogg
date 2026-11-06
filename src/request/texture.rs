use crate::client::{Nidhogg, NidhoggError};
use crate::structs::textures::ModelType;
use reqwest::multipart::{Form, Part};
use std::fmt::Display;

impl Nidhogg {
    pub async fn upload_texture(
        &self,
        unhyphenated_uuid: String,
        access_token: String,
        texture_type: TextureType,
        texture: Vec<u8>,
        file_name: String,
        model_type: Option<ModelType>,
    ) -> Result<(), NidhoggError> {
        let url = format!("/api/user/profile/{unhyphenated_uuid}/{texture_type}");

        fn model_to_string(model_type: ModelType) -> String {
            match model_type {
                ModelType::DEFAULT => "".to_string(),
                ModelType::SLIM => "slim".to_string(),
            }
        }

        let form = Form::new()
            .text("model", model_type.map_or("".to_string(), model_to_string))
            .part(
                "file",
                Part::bytes(texture)
                    .file_name(file_name)
                    .mime_str("image/png")?,
            );

        let response = self.put_request(url.as_str(), form, access_token).await?;

        match response {
            None => Ok(()),
            Some(response) => Err(NidhoggError::Other(format!(
                "Failed to upload texture: {}",
                response
            ))),
        }
    }

    pub async fn delete_texture(
        &self,
        unhyphenated_uuid: String,
        access_token: String,
        texture_type: TextureType,
    ) -> Result<(), NidhoggError> {
        let url = format!("/api/user/profile/{unhyphenated_uuid}/{texture_type}");
        self.delete_request(url.as_str(), access_token).await
    }
}

pub enum TextureType {
    SKIN,
    CAPE,
}

impl Display for TextureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TextureType::SKIN => write!(f, "skin"),
            TextureType::CAPE => write!(f, "cape"),
        }
    }
}
