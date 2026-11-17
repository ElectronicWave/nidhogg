# nidhogg
> [English](./README.md) | 简体中文

此 Rust 库提供了一个用于与 Yggdrasil 验证系统进行交互的接口。  
该库包含用于处理认证请求、响应及错误的模型，同时提供了一个客户端，用于通过 HTTP 请求执行用户认证、令牌刷新、令牌验证以及注销操作。  
**实现标准：** [authlib-injector](https://yushijinhun.github.io/authlib-injector/zh/Yggdrasil-%E6%9C%8D%E5%8A%A1%E7%AB%AF%E6%8A%80%E6%9C%AF%E8%A7%84%E8%8C%83.html)（Yggdrasil 服务器技术规范）

## 功能
- **Nidhogg Client**: 处理验证请求操作
- **JSON 模型**: 用于处理序列化/反序列化 HTTP 请求与返回，包括代理，角色信息，用户信息和错误返回的结构体
- **错误处理**: 自定义 Error 类型，用于处理验证时错误
- **离线服务器**: 可选的本地 Yggdrasil 会话/纹理服务，用于离线配置文件和本地皮肤/披风

## 离线服务器
`服务器`功能为 `authlib-injector` 提供了离线配置文件所需的端点：  
来自[HMCL](https://github.com/HMCL-dev/HMCL/blob/main/HMCLCore/src/main/java/org/jackhuang/hmcl/auth/offline/YggdrasilServer.java)  

- `GET /`
- `GET /status`
- `POST /api/profiles/minecraft`
- `GET /sessionserver/session/minecraft/hasJoined`
- `POST /sessionserver/session/minecraft/join`
- `GET /sessionserver/session/minecraft/profile/{uuid}`
- `GET /textures/{uuid}/skin`
- `GET /textures/{uuid}/cape`

它未实现 `/authserver/*` 。离线凭据由启动器生成；该服务仅向 `authlib-injector` 暴露配置文件和已签名的纹理属性。
```toml
[dependencies]
nidhogg = { version = "0.3.0", features = ["server"] }
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

启动参数:

```text
-javaagent:authlib-injector.jar=<server.base_url>
-Dauthlibinjector.side=client
```

对于局域网客户端，请通过 `public_base_url` 传递一个可访问的地址，例如 `http://192.168.1.10:<端口>`。该服务器不提供 TLS 支持；请仅将其绑定到可信网络。

## 客户端使用
```toml
[dependencies]
nidhogg = "0.3.0"
```

## 例子
> 同见 [libs.rs](./src/lib.rs)
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
