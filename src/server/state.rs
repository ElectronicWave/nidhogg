use std::{collections::HashMap, path::PathBuf, sync::Arc};

use md5::{Digest, Md5};
use url::Url;

use super::{
    OfflineProfile, OfflineServerBuilder, SkinModel, error::ServerError, signature::SigningState,
};

pub(super) struct OfflineState {
    pub(super) server_name: String,
    pub(super) base_url: Url,
    pub(super) skin_domains: Vec<String>,
    pub(super) profiles: HashMap<String, Arc<ResolvedProfile>>,
    pub(super) profiles_by_name: HashMap<String, String>,
    pub(super) signing: SigningState,
}

impl OfflineState {
    pub(super) fn build(builder: OfflineServerBuilder, base_url: Url) -> Result<Self, ServerError> {
        let profiles = resolve_profiles(builder.profiles)?;
        let mut resolved_profiles = HashMap::with_capacity(profiles.len());
        let mut profiles_by_name = HashMap::with_capacity(profiles.len());

        for profile in profiles {
            let profile = Arc::new(profile);
            profiles_by_name.insert(profile.username.to_lowercase(), profile.id.clone());
            resolved_profiles.insert(profile.id.clone(), profile);
        }

        let mut skin_domains = builder.skin_domains;
        if let Some(host) = base_url.host_str()
            && !skin_domains.iter().any(|domain| domain == host)
        {
            skin_domains.push(host.to_owned());
        }

        Ok(Self {
            server_name: builder.server_name,
            base_url,
            skin_domains,
            profiles: resolved_profiles,
            profiles_by_name,
            signing: SigningState::load(builder.signing_key_pem.as_deref())?,
        })
    }

    pub(super) fn profile_by_name(&self, username: &str) -> Option<Arc<ResolvedProfile>> {
        let profile_id = self.profiles_by_name.get(&username.to_lowercase())?;
        self.profiles.get(profile_id).cloned()
    }

    pub(super) fn profile_count(&self) -> usize {
        self.profiles.len()
    }
}

#[derive(Debug, Clone)]
pub(super) struct ResolvedProfile {
    pub(super) id: String,
    pub(super) username: String,
    pub(super) skin: Option<Arc<[u8]>>,
    pub(super) skin_model: SkinModel,
    pub(super) cape: Option<Arc<[u8]>>,
}

fn resolve_profiles(profiles: Vec<OfflineProfile>) -> Result<Vec<ResolvedProfile>, ServerError> {
    let mut usernames = HashMap::new();
    let mut profile_ids = HashMap::new();
    let mut resolved = Vec::with_capacity(profiles.len());

    for profile in profiles {
        if profile.username.trim().is_empty() {
            return Err(ServerError::EmptyUsername);
        }

        let username_key = profile.username.to_lowercase();
        if usernames
            .insert(username_key, profile.username.clone())
            .is_some()
        {
            return Err(ServerError::DuplicateUsername {
                username: profile.username,
            });
        }

        let id = match profile.uuid {
            Some(uuid) => parse_profile_uuid(uuid)?,
            None => offline_uuid(&profile.username),
        };
        if profile_ids
            .insert(id.clone(), profile.username.clone())
            .is_some()
        {
            return Err(ServerError::DuplicateUuid { uuid: id });
        }

        resolved.push(ResolvedProfile {
            id: id.clone(),
            username: profile.username,
            skin: profile.skin.map(read_texture).transpose()?,
            skin_model: profile.skin_model,
            cape: profile.cape.map(read_texture).transpose()?,
        });
    }

    Ok(resolved)
}

fn parse_profile_uuid(uuid: String) -> Result<String, ServerError> {
    let compact = uuid.replace('-', "").to_ascii_lowercase();
    if compact.len() != 32 || !compact.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ServerError::InvalidUuid {
            uuid,
            reason: "expected 32 hexadecimal digits, optionally separated by hyphens".to_owned(),
        });
    }
    Ok(compact)
}

fn offline_uuid(username: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(b"OfflinePlayer:");
    hasher.update(username.as_bytes());
    let mut bytes: [u8; 16] = hasher.finalize().into();
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    hex_encode(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

fn read_texture(path: PathBuf) -> Result<Arc<[u8]>, ServerError> {
    let bytes = std::fs::read(&path).map_err(|source| ServerError::TextureRead {
        path: path.clone(),
        source,
    })?;
    if !bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a]) {
        return Err(ServerError::InvalidTexture { path });
    }

    Ok(Arc::from(bytes))
}
