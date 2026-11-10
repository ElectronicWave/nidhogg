use serde::Deserialize;

/// (Optional) Base64-encoded JSON string containing the profile’s texture information
#[derive(Debug, Clone, Deserialize)]
pub struct TextureProperty {
    /// Timestamp when this property value was generated (Java/Unix timestamp)
    pub timestamp: i64,

    /// Profile UUID (Unhyphenated)
    #[serde(rename = "profileId")]
    pub profile_id: String,

    /// Profile name
    #[serde(rename = "profileName")]
    pub name: String,

    /// Profile textures
    pub textures: Textures,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Textures {
    #[serde(rename = "SKIN")]
    pub skin: Option<Texture>,

    #[serde(rename = "CAPE")]
    pub cape: Option<Texture>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Texture {
    /// Texture URL
    pub url: String,

    /// Texture metadata, if none, it need not be included
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Metadata {
    pub model: ModelType,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ModelType {
    /// Skin with normal arm width (4px)
    #[serde(rename = "default")]
    DEFAULT,

    /// Skin with thinner arms (3px)
    #[serde(rename = "slim")]
    SLIM,
}
