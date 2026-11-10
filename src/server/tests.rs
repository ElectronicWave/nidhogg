use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{Client, StatusCode};
use rsa::{PublicKey, pkcs1v15::Pkcs1v15Sign, pkcs8::DecodePublicKey};
use serde_json::{Value, json};
use sha1::{Digest, Sha1};

use super::{OfflineProfile, OfflineServer, SkinModel};

const PNG: &[u8] = &[
    0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
];
static NEXT_TEXTURE: AtomicU64 = AtomicU64::new(0);

#[tokio::test]
async fn offline_server_supports_local_profiles_and_textures() {
    let skin = texture_path("skin");
    let cape = texture_path("cape");
    let server = OfflineServer::builder()
        .server_name("Test Server".to_owned())
        .profile(
            OfflineProfile::new("Steve".to_owned())
                .with_skin(skin.clone())
                .with_cape(cape.clone())
                .with_skin_model(SkinModel::Slim),
        )
        .start(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .unwrap();
    let client = Client::new();

    let metadata: Value = client
        .get(server.base_url.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(metadata["meta"]["implementationName"], "nidhogg");
    assert_eq!(metadata["meta"]["serverName"], "Test Server");
    assert_eq!(
        metadata["skinDomains"],
        json!([server.base_url.host_str().unwrap()])
    );

    let profiles: Value = client
        .post(server.base_url.join("api/profiles/minecraft").unwrap())
        .json(&json!(["Steve", "Missing"]))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let profile_id = profiles[0]["id"].as_str().unwrap();
    assert_eq!(profiles.as_array().unwrap().len(), 1);
    assert_eq!(profiles[0]["name"], "Steve");

    let joined: Value = client
        .get(
            server
                .base_url
                .join("sessionserver/session/minecraft/hasJoined?username=Steve&serverId=test")
                .unwrap(),
        )
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(joined["id"], profile_id);
    assert_eq!(joined["name"], "Steve");

    let property = &joined["properties"][0];
    let value = property["value"].as_str().unwrap();
    let signature = STANDARD
        .decode(property["signature"].as_str().unwrap())
        .unwrap();
    let public_key =
        PublicKey::from_public_key_pem(metadata["signaturePublickey"].as_str().unwrap()).unwrap();
    public_key
        .verify(
            Pkcs1v15Sign::new::<Sha1>(),
            &Sha1::digest(value.as_bytes()),
            &signature,
        )
        .unwrap();

    let textures: Value = serde_json::from_slice(&STANDARD.decode(value).unwrap()).unwrap();
    assert_eq!(textures["profileId"], profile_id);
    assert_eq!(textures["profileName"], "Steve");
    assert_eq!(textures["textures"]["SKIN"]["metadata"]["model"], "slim");
    assert_eq!(
        textures["textures"]["SKIN"]["url"],
        server
            .base_url
            .join(&format!("textures/{profile_id}/skin"))
            .unwrap()
            .as_str()
    );
    assert_eq!(
        textures["textures"]["CAPE"]["url"],
        server
            .base_url
            .join(&format!("textures/{profile_id}/cape"))
            .unwrap()
            .as_str()
    );

    let join = client
        .post(
            server
                .base_url
                .join("sessionserver/session/minecraft/join")
                .unwrap(),
        )
        .json(&json!({
            "accessToken": "offline",
            "selectedProfile": profile_id,
            "serverId": "test"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(join.status(), StatusCode::NO_CONTENT);

    let profile_url = server
        .base_url
        .join(&format!(
            "sessionserver/session/minecraft/profile/{profile_id}"
        ))
        .unwrap();
    let unsigned: Value = client
        .get(format!("{profile_url}?unsigned=true"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(unsigned["properties"][0].get("signature").is_none());

    let default: Value = client
        .get(profile_url.clone())
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(default["properties"][0].get("signature").is_none());

    let signed: Value = client
        .get(format!("{profile_url}?unsigned=false"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(signed["properties"][0].get("signature").is_some());

    let skin_response = client
        .get(
            server
                .base_url
                .join(&format!("textures/{profile_id}/skin"))
                .unwrap(),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(skin_response.status(), StatusCode::OK);
    assert_eq!(skin_response.bytes().await.unwrap().as_ref(), PNG);

    server.shutdown().unwrap();
    std::fs::remove_file(skin).unwrap();
    std::fs::remove_file(cape).unwrap();
}

fn texture_path(name: &str) -> PathBuf {
    let sequence = NEXT_TEXTURE.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!(
        "nidhogg-{name}-{}-{sequence}.png",
        std::process::id()
    ));
    std::fs::write(&path, PNG).unwrap();
    path
}
