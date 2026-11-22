use crate::error::Result;
use crate::types::meta::Meta;
use reqwest::header::{ACCEPT_CHARSET, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::{Client, RequestBuilder};
use serde::Serialize;

pub struct Nidhogg {
    pub yggdrasil_url: String,
    pub(crate) client: Client,
}

const USER_AGENT: &str = concat!("nidhogg/", env!("CARGO_PKG_VERSION"));

impl Nidhogg {
    pub fn new(yggdrasil_url: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(ACCEPT_CHARSET, HeaderValue::from_static("UTF-8"));

        let client = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()?;

        Ok(Self::with_client(yggdrasil_url, client))
    }

    pub fn with_client(yggdrasil_url: &str, client: Client) -> Self {
        Nidhogg {
            yggdrasil_url: yggdrasil_url.to_string(),
            client,
        }
    }

    pub(crate) fn post_json<T: Serialize + ?Sized>(&self, path: &str, body: &T) -> RequestBuilder {
        self.client
            .post(format!("{}{}", self.yggdrasil_url, path))
            .header(ACCEPT_CHARSET, HeaderValue::from_static("UTF-8"))
            .json(body)
    }

    pub async fn metadata(&self) -> Result<Meta> {
        Ok(self
            .client
            .get(format!("{}/", self.yggdrasil_url))
            .header(ACCEPT_CHARSET, HeaderValue::from_static("UTF-8"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    use crate::client::Nidhogg;
    use crate::request::validate::Tokens;

    #[tokio::test]
    async fn with_client_uses_custom_client_and_sends_json_headers() {
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
            stream
                .write_all(
                    b"HTTP/1.1 204 No Content\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .await
                .expect("write response");
            String::from_utf8_lossy(&data).into_owned()
        });

        let mut headers = HeaderMap::new();
        headers.insert(
            HeaderName::from_static("x-custom-client"),
            HeaderValue::from_static("nidhogg-test"),
        );
        let custom_client = reqwest::Client::builder()
            .no_proxy()
            .default_headers(headers)
            .build()
            .expect("build custom client");

        let nidhogg = Nidhogg::with_client(&format!("http://{addr}"), custom_client);
        nidhogg
            .validate(&Tokens::new("token".to_owned(), None))
            .await
            .expect("validate succeeds");

        let request = server.await.expect("join server").to_ascii_lowercase();
        assert!(request.contains("x-custom-client: nidhogg-test"));
        assert!(request.contains("accept-charset: utf-8"));
        assert!(request.contains("content-type: application/json"));
    }
}
