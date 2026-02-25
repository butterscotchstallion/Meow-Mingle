mod config_loader;

use axum::{Router, Json, routing::get};
use serde::Serialize;
use auth_framework::{AuthFramework, AuthConfig, JwtMethod};
use auth_framework::methods::AuthMethodEnum;
use crate::config_loader::{load_config, parse_expiry};

#[derive(Serialize)]
struct Greeting {
    message: String,
}

async fn hello() -> Json<Greeting> {
    Json(Greeting {
        message: "Hello, world!".into(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app_config = load_config()?;

    println!("Loaded config: {:#?}", app_config);

    let auth_config = AuthConfig {
        token_lifetime: parse_expiry(&app_config.jwt.expiry),
        ..AuthConfig::new()
    };

    let mut auth = AuthFramework::new(auth_config);

    let jwt_method = JwtMethod::new()
        .secret_key(&app_config.jwt.secret_key)
        .issuer("meow-mingle");

    auth.register_method("jwt", AuthMethodEnum::Jwt(jwt_method));
    auth.initialize().await?;

    println!("JWT algorithm: {}", app_config.jwt.algorithm);
    println!("Session domain: {}", app_config.session.domain);
    println!("✅  Auth framework initialized with JWT method");

    // API server
    let app = Router::new().route("/hello", get(hello));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await?;

    axum::serve(listener, app).await?;

    Ok(())
}