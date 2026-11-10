use base64::{Engine, engine::general_purpose::STANDARD};
use rsa::{
    Generate, PrivateKey, PublicKey,
    pkcs1::DecodeRsaPrivateKey,
    pkcs8::{DecodePrivateKey, EncodePublicKey, LineEnding},
};
use sha1::Sha1;

use super::error::ServerError;

pub(super) struct SigningState {
    key: PrivateKey,
    public_key_pem: String,
}

impl SigningState {
    pub(super) fn load(pem: Option<&str>) -> Result<Self, ServerError> {
        let private_key = match pem {
            Some(pem) => parse_private_key(pem)?,
            None => PrivateKey::generate(),
        };
        let public_key = PublicKey::from(&private_key);
        let public_key_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|error| ServerError::InvalidSigningKey {
                reason: error.to_string(),
            })?;

        Ok(Self {
            key: private_key,
            public_key_pem,
        })
    }

    pub(super) fn public_key_pem(&self) -> &str {
        &self.public_key_pem
    }

    pub(super) fn sign(&self, value: &str) -> Result<String, ServerError> {
        let signature = self
            .key
            .pkcs1v15_sign::<Sha1>(value.as_bytes())
            .map_err(|error| ServerError::Sign {
                reason: error.to_string(),
            })?;
        Ok(STANDARD.encode(signature))
    }
}

fn parse_private_key(pem: &str) -> Result<PrivateKey, ServerError> {
    let pkcs8_error = match PrivateKey::from_pkcs8_pem(pem) {
        Ok(key) => return Ok(key),
        Err(error) => error,
    };
    let pkcs1_error = match PrivateKey::from_pkcs1_pem(pem) {
        Ok(key) => return Ok(key),
        Err(error) => error,
    };

    Err(ServerError::InvalidSigningKey {
        reason: format!("PKCS#8: {pkcs8_error}; PKCS#1: {pkcs1_error}"),
    })
}
