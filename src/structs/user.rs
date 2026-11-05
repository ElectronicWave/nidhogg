use serde::Deserialize;
use crate::structs::Property;

pub const PREFERRED_LANGUAGE_PROPERTY_NAME: &str = "preferredLanguage";

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    /// User id
    pub id: String,

    /// User properties
    /// Available property:
    /// - preferredLanguage: (optional) user's preferred language, e.g. `en`, `zh_CN`
    pub properties: Vec<Property>
}

impl User {
    /// Get the user's preferred language if existed.
    pub fn get_preferred_language(&self) -> Option<&String> {
        self.properties.iter()
            .find(|p| p.name == PREFERRED_LANGUAGE_PROPERTY_NAME)
            .map(|p| &p.value)
    }
}