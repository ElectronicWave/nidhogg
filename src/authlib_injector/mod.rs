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

        Ok(Self::with_client(base_url, client))
    }

    pub fn with_client(base_url: &str, client: Client) -> Self {
        Self {
            base_url: base_url.to_owned(),
            client,
        }
    }

    pub fn official() -> Result<Self, NidhoggError> {
        Self::new(OFFICIAL_URL)
    }

    pub fn official_with_client(client: Client) -> Self {
        Self::with_client(OFFICIAL_URL, client)
    }

    pub fn bmclapi() -> Result<Self, NidhoggError> {
        Self::new(BMCLAPI_URL)
    }

    pub fn bmclapi_with_client(client: Client) -> Self {
        Self::with_client(BMCLAPI_URL, client)
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

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use super::Distribution;

    #[tokio::test]
    async fn with_client_uses_custom_client_for_latest() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");

        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut buffer = vec![0u8; 4096];
            let mut data = Vec::new();
            loop {
                let read = stream.read(&mut buffer).await.expect("read request");
                if read == 0 {
                    break;
                }
                data.extend_from_slice(&buffer[..read]);
                if data.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let body = r#"{"build_number":1,"version":"1.2.5","release_time":"2026-01-01T00:00:00Z","download_url":"https://example.com/ai.jar","checksums":{"sha256":"abc"}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write response");
            String::from_utf8_lossy(&data).into_owned()
        });

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-custom-dist"),
            HeaderValue::from_static("dist-test"),
        );
        let custom_client = reqwest::Client::builder()
            .no_proxy()
            .default_headers(headers)
            .build()
            .expect("build custom client");

        let dist = Distribution::with_client(&format!("http://{addr}"), custom_client);
        let artifact = dist.latest().await.expect("fetch latest");
        assert_eq!(artifact.version, "1.2.5");

        let request = server.await.expect("join server").to_ascii_lowercase();
        assert!(request.contains("x-custom-dist: dist-test"));
    }
}
