use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;
use serde_json::{Map, Value, json};

use crate::types::{Property, profile::Profile};

use super::{
    SkinModel,
    error::ServerError,
    http::{Method, Request, Response},
    state::{OfflineState, ResolvedProfile},
};

pub(super) fn handle(state: &OfflineState, request: &Request) -> Result<Response, ServerError> {
    match (request.method, request.path.as_str()) {
        (Method::Get, "/") => metadata(state),
        (Method::Get, "/status") => status(state),
        (Method::Post, "/api/profiles/minecraft") => profiles(state, request),
        (Method::Get, "/sessionserver/session/minecraft/hasJoined") => has_joined(state, request),
        (Method::Post, "/sessionserver/session/minecraft/join") => Ok(Response::empty(204)),
        (Method::Get, path) if path.starts_with("/sessionserver/session/minecraft/profile/") => {
            profile(state, request, path)
        }
        (Method::Get, path) if path.starts_with("/textures/") => texture(state, path),
        _ => Ok(Response::empty(404)),
    }
}

#[derive(Serialize)]
struct MetadataResponse {
    meta: MetadataMeta,
    #[serde(rename = "skinDomains")]
    skin_domains: Vec<String>,
    #[serde(rename = "signaturePublickey")]
    signature_public_key: String,
}

#[derive(Serialize)]
struct MetadataMeta {
    #[serde(rename = "serverName")]
    server_name: String,
    #[serde(rename = "implementationName")]
    implementation_name: &'static str,
    #[serde(rename = "implementationVersion")]
    implementation_version: &'static str,
}

fn metadata(state: &OfflineState) -> Result<Response, ServerError> {
    Response::json(&MetadataResponse {
        meta: MetadataMeta {
            server_name: state.server_name.clone(),
            implementation_name: "nidhogg",
            implementation_version: env!("CARGO_PKG_VERSION"),
        },
        skin_domains: state.skin_domains.clone(),
        signature_public_key: state.signing.public_key_pem().to_owned(),
    })
}

fn status(state: &OfflineState) -> Result<Response, ServerError> {
    Response::json(&json!({
        "user.count": state.profile_count(),
        "token.count": 0,
        "pendingAuthentication.count": 0,
    }))
}

fn profiles(state: &OfflineState, request: &Request) -> Result<Response, ServerError> {
    let names: Vec<String> = match serde_json::from_slice(&request.body) {
        Ok(names) => names,
        Err(_) => return Ok(Response::text(400, "invalid JSON profile name list")),
    };
    let profiles = names
        .iter()
        .filter_map(|name| state.profile_by_name(name))
        .map(|profile| profile_to_response(&profile))
        .collect::<Vec<_>>();

    Response::json(&profiles)
}

fn has_joined(state: &OfflineState, request: &Request) -> Result<Response, ServerError> {
    let Some(username) = request.query_value("username") else {
        return Ok(Response::text(400, "missing username"));
    };
    let Some(profile) = state.profile_by_name(username) else {
        return Ok(Response::empty(204));
    };

    complete_profile(state, &profile, false)
}

fn profile(state: &OfflineState, request: &Request, path: &str) -> Result<Response, ServerError> {
    let uuid = &path["/sessionserver/session/minecraft/profile/".len()..];
    let Some(profile) = find_profile(state, uuid) else {
        return Ok(Response::empty(204));
    };
    let unsigned = request.query_value("unsigned") != Some("false");

    complete_profile(state, &profile, unsigned)
}

fn texture(state: &OfflineState, path: &str) -> Result<Response, ServerError> {
    let Some(path) = path.strip_prefix("/textures/") else {
        return Ok(Response::empty(404));
    };
    let mut parts = path.split('/');
    let Some(uuid) = parts.next() else {
        return Ok(Response::empty(404));
    };
    let Some(kind) = parts.next() else {
        return Ok(Response::empty(404));
    };
    if parts.next().is_some() {
        return Ok(Response::empty(404));
    }

    let Some(profile) = find_profile(state, uuid) else {
        return Ok(Response::empty(404));
    };
    let texture = match kind {
        "skin" => profile.skin.as_deref(),
        "cape" => profile.cape.as_deref(),
        _ => return Ok(Response::empty(404)),
    };

    match texture {
        Some(texture) => Ok(Response::png(texture)),
        None => Ok(Response::empty(404)),
    }
}

fn complete_profile(
    state: &OfflineState,
    profile: &ResolvedProfile,
    unsigned: bool,
) -> Result<Response, ServerError> {
    let properties = textures_property(state, profile, unsigned)?
        .map(|property| vec![property])
        .unwrap_or_default();

    Response::json(&Profile {
        id: profile.id.clone(),
        name: profile.username.clone(),
        properties: Some(properties),
    })
}

fn find_profile(state: &OfflineState, uuid: &str) -> Option<Arc<ResolvedProfile>> {
    let uuid = uuid.replace('-', "").to_ascii_lowercase();
    state.profiles.get(&uuid).cloned()
}

fn profile_to_response(profile: &ResolvedProfile) -> Profile {
    Profile {
        id: profile.id.clone(),
        name: profile.username.clone(),
        properties: None,
    }
}

fn textures_property(
    state: &OfflineState,
    profile: &ResolvedProfile,
    unsigned: bool,
) -> Result<Option<Property>, ServerError> {
    if profile.skin.is_none() && profile.cape.is_none() {
        return Ok(None);
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ServerError::Clock)?
        .as_millis();
    let mut textures = Map::new();

    if profile.skin.is_some() {
        let mut skin = Map::new();
        skin.insert(
            "url".to_owned(),
            Value::String(texture_url(&state.base_url, &profile.id, "skin")),
        );
        if profile.skin_model == SkinModel::Slim {
            skin.insert("metadata".to_owned(), json!({ "model": "slim" }));
        }
        textures.insert("SKIN".to_owned(), Value::Object(skin));
    }

    if profile.cape.is_some() {
        textures.insert(
            "CAPE".to_owned(),
            json!({ "url": texture_url(&state.base_url, &profile.id, "cape") }),
        );
    }

    let payload = json!({
        "timestamp": timestamp,
        "profileId": profile.id,
        "profileName": profile.username,
        "textures": textures,
    });
    let value = STANDARD.encode(serde_json::to_vec(&payload)?);
    let signature = match unsigned {
        true => None,
        false => Some(state.signing.sign(&value)?),
    };

    Ok(Some(Property {
        name: "textures".to_owned(),
        value,
        signature,
    }))
}

fn texture_url(base_url: &url::Url, uuid: &str, kind: &str) -> String {
    let mut url = base_url.clone();
    url.set_path(&format!("/textures/{uuid}/{kind}"));
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}
