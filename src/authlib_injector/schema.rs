use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub build_number: u64,
    pub version: String,
    pub release_time: String,
    pub download_url: String,
    pub checksums: Checksums,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Checksums {
    pub sha256: String,
}
