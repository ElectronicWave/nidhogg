pub mod client;
pub mod error;
pub mod request;
pub mod structs;

#[cfg(test)]
mod tests {
    use crate::client::Nidhogg;
    use crate::error::NidhoggError;
    use crate::request::authenticate::Authenticate;

    const MOCK_API_ROOT: &str = "https://auth-demo.yushi.moe";

    #[tokio::test]
    async fn get_metadata() -> Result<(), NidhoggError> {
        let nidhogg = Nidhogg::new(MOCK_API_ROOT)?;
        println!("{:?}", nidhogg.metadata().await?);
        assert_eq!(
            nidhogg
                .metadata()
                .await?
                .meta
                .implementation_name
                .unwrap()
                .as_str(),
            "yggdrasil-mock-server"
        );
        Ok(())
    }

    #[tokio::test]
    async fn authenticate() -> Result<(), NidhoggError> {
        let nidhogg = Nidhogg::new(MOCK_API_ROOT)?;
        let authenticate_response = nidhogg
            .authenticate(&Authenticate::new(
                "test1@example.com".to_string(),
                "111111".to_string(),
                None,
                false,
            ))
            .await?;
        println!("{:?}", &authenticate_response);
        Ok(())
    }
}
