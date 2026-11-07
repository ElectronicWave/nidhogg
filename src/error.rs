use thiserror::Error;

#[derive(Debug, Error)]
pub enum NidhoggError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("failed to parse JSON: {0}")]
    Json(#[from] serde_json::Error),
}
