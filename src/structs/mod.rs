use serde::Deserialize;

pub mod user;
pub mod profile;
pub mod textures;

#[derive(Debug, Clone, Deserialize)]
pub struct Property {
    /// Property name
    pub name: String,

    /// Property value
    pub value: String,

    /// Digital signature (optional)
    /// Format: `Base64`
    /// Algorithm: `SHA1withRSA`
    pub signature: Option<String>
}