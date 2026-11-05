use std::fmt::Display;
use serde::Deserialize;
use thiserror::Error;

/// API Error message result
/// ```json
/// {
///     "error": "description of the error (machine-readable)",
///     "errorMessage": "detail message (human-readable)",
///     "cause": "error cause (optional)"
/// }
/// ```
#[derive(Deserialize, Error, Debug)]
pub struct Error {
    pub error: String,
    #[serde(rename = "errorMessage")]
    pub error_msg: String,
    pub cause: Option<String>,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}, Message: {}, Cause: {:?}", self.error, self.error_msg, self.cause)
    }
}