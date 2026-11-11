#[cfg(feature = "authlib-injector-embed")]
mod embedded;
mod schema;

use std::time::Duration;

use reqwest::Client;

#[cfg(feature = "authlib-injector-embed")]
pub use embedded::{EmbeddedArtifact, embedded};
pub use schema::{Artifact, Checksums};

use crate::error::NidhoggError;

pub const OFFICIAL_URL: &str = "https://authlib-injector.yushi.moe";
pub const BMCLAPI_URL: &str = "https://bmclapi2.bangbang93.com/mirrors/authlib-injector";

/// Where the resolved authlib-injector artifact came from.
#[cfg(feature = "authlib-injector-embed")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactSource {
    /// Fresh metadata fetched from the distribution endpoint.
    Latest(Artifact),

    /// The build-time snapshot, including the matching JAR bytes.
    Embedded(EmbeddedArtifact),
}

pub struct Distribution {
    base_url: String,
    client: Client,
}

impl Distribution {
    pub fn new(base_url: &str) -> Result<Self, NidhoggError> {
        let client = Client::builder()
            .user_agent(concat!("nidhogg/", env!("CARGO_PKG_VERSION")))
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            base_url: base_url.to_owned(),
            client,
        })
    }

    pub fn official() -> Result<Self, NidhoggError> {
        Self::new(OFFICIAL_URL)
    }

    pub fn bmclapi() -> Result<Self, NidhoggError> {
        Self::new(BMCLAPI_URL)
    }

    pub async fn latest(&self) -> Result<Artifact, NidhoggError> {
        let body = self
            .client
            .get(format!("{}/artifact/latest.json", self.base_url))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?;
        Ok(serde_json::from_str(&body)?)
    }

    #[cfg(feature = "authlib-injector-embed")]
    pub async fn latest_or_embedded(&self) -> Result<ArtifactSource, NidhoggError> {
        match self.latest().await {
            Ok(artifact) => Ok(ArtifactSource::Latest(artifact)),
            Err(NidhoggError::Reqwest(_)) => Ok(ArtifactSource::Embedded(embedded::embedded()?)),
            Err(error) => Err(error),
        }
    }
}
