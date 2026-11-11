use std::sync::OnceLock;

use super::Artifact;
use crate::error::NidhoggError;

const LATEST: &str = include_str!(concat!(env!("OUT_DIR"), "/latest.json"));
const JAR: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/latest.jar"));

static ARTIFACT: OnceLock<Artifact> = OnceLock::new();

/// The build-time snapshot paired with the JAR bytes it describes.
#[derive(Clone, PartialEq, Eq)]
pub struct EmbeddedArtifact {
    artifact: Artifact,
    jar: &'static [u8],
}

impl EmbeddedArtifact {
    pub fn artifact(&self) -> &Artifact {
        &self.artifact
    }

    pub fn jar(&self) -> &'static [u8] {
        self.jar
    }
}

impl std::fmt::Debug for EmbeddedArtifact {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("EmbeddedArtifact")
            .field("artifact", &self.artifact)
            .field("jar_len", &self.jar.len())
            .finish()
    }
}

pub fn embedded() -> Result<EmbeddedArtifact, NidhoggError> {
    let artifact = match ARTIFACT.get() {
        Some(artifact) => artifact.clone(),
        None => {
            let parsed = serde_json::from_str::<Artifact>(LATEST)?;
            ARTIFACT.get_or_init(|| parsed).clone()
        }
    };

    Ok(EmbeddedArtifact { artifact, jar: JAR })
}
