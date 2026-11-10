# nidhogg
This Rust library provides an interface for interacting with Yggdrasil's authentication system. It includes models for handling authentication requests, responses, and errors, as well as a client for sending HTTP requests to authenticate, refresh, validate, and sign out users.  
Implemented Standard: [authlib-injector](https://yushijinhun.github.io/authlib-injector/en/yggdrasil-server-technical-specification.html)

## Features
- **Nidhogg Client**: Handling authentication request operations.
- **JSON Models**: Structs for serializing/deserializing request and response data, including agents, profiles, users, and errors.
- **Error Handling**: Custom error types for handling authentication failures.
- **Offline Server**: Optional local Yggdrasil session/texture service for offline profiles and local skins/capes.

## Offline Server
The `server` feature provides the endpoints authlib-injector needs for an offline profile:

The offline server endpoint layout and behavior are referenced from
[HMCL](https://github.com/HMCL-dev/HMCL/blob/main/HMCLCore/src/main/java/org/jackhuang/hmcl/auth/offline/YggdrasilServer.java).

- `GET /`
- `GET /status`
- `POST /api/profiles/minecraft`
- `GET /sessionserver/session/minecraft/hasJoined`
- `POST /sessionserver/session/minecraft/join`
- `GET /sessionserver/session/minecraft/profile/{uuid}`
- `GET /textures/{uuid}/skin`
- `GET /textures/{uuid}/cape`

It does not implement `/authserver/*`. Offline credentials are constructed by the launcher; this service only exposes profiles and signed texture properties to authlib-injector.

```toml
[dependencies]
nidhogg = { version = "0.2.0", features = ["server"] }
```

```rust
use std::net::{Ipv4Addr, SocketAddr};

use nidhogg::server::{OfflineProfile, OfflineServer};

let server = OfflineServer::builder()
    .server_name("My Server".to_owned())
    .profile(
        OfflineProfile::new("Steve".to_owned())
            .with_skin("skin.png".into())
            .with_cape("cape.png".into()),
    )
    .start(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))?;

println!("authlib-injector server: {}", server.base_url);
server.shutdown()?;
```

Launch arguments:

```text
-javaagent:authlib-injector.jar=<server.base_url>
-Dauthlibinjector.side=client
```

For LAN clients, pass a reachable address with `public_base_url`, for example `http://192.168.1.10:<port>`. The server does not provide TLS; bind it to a trusted network only.

## Usage
To use the `nidhogg` library, add it as a dependency in your `Cargo.toml` file:
```toml
[dependencies]
nidhogg = "0.1.0"
```

## Sample Code
> See also [libs.rs](./src/lib.rs) tests crate.
```rust
use std::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let nidhogg = Nidhogg::new(
        "https://myauthserver.com/auth-endpoint".to_string(), // No "/" at the end
        // Native Yggdrasil server support coming soon
    );

    // Authenticate a user
    let auth = nidhogg
        .authenticate(Authenticate::new(
            "Username",
            "Password",
            "ClientToken",
            true, // Request user info
        ))
        .await?;

    println!("Access Token: {}", auth.access_token);

    // Refresh token
    let refresh = nidhogg
        .refresh(Refresh::new(
            &auth.access_token,
            &auth.client_token,
            true, // Request user info
            None, // No selected profile
        ))
        .await?;

    println!("New Access Token: {}", refresh_response.access_token);

    // Validate token
    if nidhogg.validate(Tokens::new(
        &refresh.access_token,
        &refresh.client_token
    )).await.is_ok() {
        println!("Token is valid");
    } else {
        println!("Token is invalid");
    }

    // Invalidate token
    nidhogg.invalidate(Tokens::new(
        &refresh.access_token,
        &refresh.client_token
    )).await?;

    // Signout
    nidhogg.signout(Signout::new(
        "Username",
        "Password"
    )).await?;

    // Get profile by UUID
    let profile = nidhogg.get_profile(
        "UUID", false
    ).await?;

    println!("Profile Name: {}", profile.name);

    // Upload texture
    let texture = nidhogg.upload_texture(
        "UUID",
        &refresh.access_token,
        TextureType::Skin,
        fs::read("skin.png")?,
        "sha256 hash_of_skin_file",
        ModelType::SLIM
    ).await?;

    Ok(())
}
```
