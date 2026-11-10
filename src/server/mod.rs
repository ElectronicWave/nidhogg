mod error;
mod http;
mod routes;
mod signature;
mod state;

#[cfg(test)]
mod tests;

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use url::Url;

pub use error::ServerError;

use state::OfflineState;

#[derive(Debug, Clone)]
pub struct OfflineProfile {
    pub username: String,
    pub uuid: Option<String>,
    pub skin: Option<PathBuf>,
    pub cape: Option<PathBuf>,
    pub skin_model: SkinModel,
}

impl OfflineProfile {
    pub fn new(username: String) -> Self {
        Self {
            username,
            uuid: None,
            skin: None,
            cape: None,
            skin_model: SkinModel::Classic,
        }
    }

    pub fn with_uuid(mut self, uuid: String) -> Self {
        self.uuid = Some(uuid);
        self
    }

    pub fn with_skin(mut self, skin: PathBuf) -> Self {
        self.skin = Some(skin);
        self
    }

    pub fn with_cape(mut self, cape: PathBuf) -> Self {
        self.cape = Some(cape);
        self
    }

    pub fn with_skin_model(mut self, skin_model: SkinModel) -> Self {
        self.skin_model = skin_model;
        self
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SkinModel {
    #[default]
    Classic,
    Slim,
}

#[derive(Debug, Default)]
pub struct OfflineServer;

impl OfflineServer {
    pub fn builder() -> OfflineServerBuilder {
        OfflineServerBuilder::new()
    }
}

pub struct OfflineServerBuilder {
    server_name: String,
    public_base_url: Option<Url>,
    signing_key_pem: Option<String>,
    skin_domains: Vec<String>,
    profiles: Vec<OfflineProfile>,
}

impl OfflineServerBuilder {
    pub fn new() -> Self {
        Self {
            server_name: "nidhogg".to_owned(),
            public_base_url: None,
            signing_key_pem: None,
            skin_domains: Vec::new(),
            profiles: Vec::new(),
        }
    }

    pub fn server_name(mut self, server_name: String) -> Self {
        self.server_name = server_name;
        self
    }

    pub fn profile(mut self, profile: OfflineProfile) -> Self {
        self.profiles.push(profile);
        self
    }

    pub fn profiles(mut self, profiles: impl IntoIterator<Item = OfflineProfile>) -> Self {
        self.profiles.extend(profiles);
        self
    }

    pub fn public_base_url(mut self, base_url: Url) -> Self {
        self.public_base_url = Some(base_url);
        self
    }

    pub fn signing_key_pem(mut self, pem: String) -> Self {
        self.signing_key_pem = Some(pem);
        self
    }

    pub fn skin_domain(mut self, domain: String) -> Self {
        self.skin_domains.push(domain);
        self
    }

    pub fn start(self, addr: SocketAddr) -> Result<RunningOfflineServer, ServerError> {
        if let Some(base_url) = &self.public_base_url
            && (base_url.path() != "/"
                || base_url.query().is_some()
                || base_url.fragment().is_some())
        {
            return Err(ServerError::InvalidBaseUrl {
                url: base_url.clone(),
            });
        }

        let listener =
            TcpListener::bind(addr).map_err(|source| ServerError::Bind { addr, source })?;
        let local_addr = listener
            .local_addr()
            .map_err(|source| ServerError::Bind { addr, source })?;
        let base_url = match self.public_base_url.clone() {
            Some(base_url) => base_url,
            None => {
                let display_addr = loopback_if_unspecified(local_addr);
                Url::parse(&format!("http://{display_addr}")).map_err(|source| {
                    ServerError::BaseUrl {
                        addr: display_addr,
                        source,
                    }
                })?
            }
        };
        let state = Arc::new(OfflineState::build(self, base_url.clone())?);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let task = thread::Builder::new()
            .name("nidhogg-offline-server".to_owned())
            .spawn(move || http::serve(listener, state, thread_stop))
            .map_err(|source| ServerError::Thread { source })?;

        Ok(RunningOfflineServer {
            addr: local_addr,
            base_url,
            stop,
            task: Some(task),
        })
    }
}

impl Default for OfflineServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RunningOfflineServer {
    pub addr: SocketAddr,
    pub base_url: Url,
    stop: Arc<AtomicBool>,
    task: Option<JoinHandle<Result<(), ServerError>>>,
}

impl RunningOfflineServer {
    pub fn shutdown(mut self) -> Result<(), ServerError> {
        self.stop();
        self.join()
    }

    fn stop(&self) {
        self.stop.store(true, Ordering::Release);
        let _ = TcpStream::connect_timeout(
            &loopback_if_unspecified(self.addr),
            Duration::from_millis(500),
        );
    }

    fn join(&mut self) -> Result<(), ServerError> {
        match self.task.take() {
            Some(task) => task.join().map_err(|_| ServerError::ThreadPanicked)?,
            None => Ok(()),
        }
    }
}

impl Drop for RunningOfflineServer {
    fn drop(&mut self) {
        self.stop();
        let _ = self.join();
    }
}

fn loopback_if_unspecified(addr: SocketAddr) -> SocketAddr {
    if !addr.ip().is_unspecified() {
        return addr;
    }

    match addr.ip() {
        IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), addr.port()),
        IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), addr.port()),
    }
}
