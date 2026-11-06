# nidhogg
This Rust library provides an interface for interacting with Yggdrasil's authentication system. It includes models for handling authentication requests, responses, and errors, as well as a client for sending HTTP requests to authenticate, refresh, validate, and sign out users.  
Implemented Standard: [authlib-injector](https://yushijinhun.github.io/authlib-injector/en/yggdrasil-server-technical-specification.html)

## Features
- **Nidhogg Client**: Handling authentication request operations.
- **JSON Models**: Structs for serializing/deserializing request and response data, including agents, profiles, users, and errors.
- **Error Handling**: Custom error types for handling authentication failures.

## Usage
To use the `nidhogg` library, add it as a dependency in your `Cargo.toml` file:
```toml
[dependencies]
nidhogg = "0.1.0"
```

## Sample Code

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