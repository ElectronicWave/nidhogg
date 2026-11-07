use crate::client::Nidhogg;
use crate::error::NidhoggError;
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

        let model = match model_type {
            Some(ModelType::SLIM) => "slim".to_string(),
            _ => "".to_string(),
        };

        let form = Form::new().text("model", model).part(
            "file",
            Part::bytes(texture)
                .file_name(file_name)
                .mime_str("image/png")?,
        );

        self.client
            .put(format!("{}{}", self.yggdrasil_url, url))
            .bearer_auth(access_token)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn delete_texture(
        &self,
        unhyphenated_uuid: String,
        access_token: String,
        texture_type: TextureType,
    ) -> Result<(), NidhoggError> {
        let url = format!("/api/user/profile/{unhyphenated_uuid}/{texture_type}");
        self.client
            .delete(format!("{}{}", self.yggdrasil_url, url))
            .bearer_auth(access_token)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
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
