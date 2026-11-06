use serde::{Deserialize, Serialize};

pub mod meta;
pub mod profile;
pub mod textures;
pub mod user;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Property {
    /// Property name
    pub name: String,

    /// Property value
    pub value: String,

    /// Digital signature (optional)
    /// Format: `Base64`
    /// Algorithm: `SHA1withRSA`
    pub signature: Option<String>,
}
