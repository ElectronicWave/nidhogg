use crate::client::NidhoggError;
use crate::client::NidhoggError::{ApiError, Other};
use serde::Deserialize;
use std::fmt::Display;
use thiserror::Error;

pub fn error(response: String) -> NidhoggError {
    let error: Result<Error, serde_json::Error> = serde_json::from_str(&response);
    match error {
        Ok(error) => ApiError(error),
        Err(_error) => Other(format!("Response: {}", response)),
    }
}

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
        write!(
            f,
            "Error: {}, Message: {}, Cause: {:?}",
            self.error, self.error_msg, self.cause
        )
    }
}
