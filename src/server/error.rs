use std::{io, net::SocketAddr, path::PathBuf};

use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("failed to bind the offline Yggdrasil server to {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: io::Error,
    },

    #[error("failed to build the base URL for {addr}: {source}")]
    BaseUrl {
        addr: SocketAddr,
        #[source]
        source: ParseError,
    },

    #[error("public base URL {url} must not contain a path, query, or fragment")]
    InvalidBaseUrl { url: Url },

    #[error("failed to read texture {path}: {source}")]
    TextureRead {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("texture {path} is not a PNG image")]
    InvalidTexture { path: PathBuf },

    #[error("profile username must not be empty")]
    EmptyUsername,

    #[error("profile username {username:?} is duplicated")]
    DuplicateUsername { username: String },

    #[error("profile UUID {uuid:?} is duplicated")]
    DuplicateUuid { uuid: String },

    #[error("invalid profile UUID {uuid:?}: {reason}")]
    InvalidUuid { uuid: String, reason: String },

    #[error("invalid signing key: {reason}")]
    InvalidSigningKey { reason: String },

    #[error("failed to sign textures: {reason}")]
    Sign { reason: String },

    #[error("failed to {context}: {source}")]
    Http {
        context: &'static str,
        #[source]
        source: io::Error,
    },

    #[error("invalid HTTP request: {reason}")]
    BadRequest { reason: String },

    #[error("HTTP request exceeds the configured size limit")]
    RequestTooLarge,

    #[error("failed to start the offline Yggdrasil server thread: {source}")]
    Thread {
        #[source]
        source: io::Error,
    },

    #[error("offline Yggdrasil server thread panicked")]
    ThreadPanicked,

    #[error("system clock is before the Unix epoch")]
    Clock,

    #[error("failed to serialize response: {0}")]
    Serialize(#[from] serde_json::Error),
}

impl ServerError {
    pub(super) fn is_client_disconnect(&self) -> bool {
        let Self::Http { source, .. } = self else {
            return false;
        };

        matches!(
            source.kind(),
            io::ErrorKind::BrokenPipe
                | io::ErrorKind::ConnectionAborted
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::TimedOut
                | io::ErrorKind::UnexpectedEof
                | io::ErrorKind::WouldBlock
        )
    }
}
