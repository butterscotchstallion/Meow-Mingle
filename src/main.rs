mod config_loader;

use crate::config_loader::get_dsn;
use auth_framework::prelude::*;
use auth_framework::AuthFramework;
use axum::{
    extract::State, http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::{get, post},
    Json,
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("auth_framework=debug,axum_integration=info,tower_http=debug")
        .init();

    let auth = create_auth_framework().await?;

    // API server
    let app = create_app(auth).await?;
    println!("✅ Web application routes configured");

    // Start the server
    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .map_err(|e| AuthError::internal(format!("Failed to bind server: {}", e)))?;

    println!("\n🚀 Server starting on http://127.0.0.1:3000");
    println!("📋 Available endpoints:");
    println!("   GET  /              - Public welcome page");
    println!("   POST /auth/login    - Login endpoint");
    println!("   GET  /auth/profile  - Get user profile (requires auth)");
    println!("   POST /auth/logout   - Logout endpoint (requires auth)");
    println!("   GET  /protected     - Protected content (requires auth)");
    println!("   GET  /admin         - Admin only content (requires admin role)");
    println!("\n💡 Try these commands:");
    println!("   curl http://127.0.0.1:3000/");
    println!(
        "   curl -X POST http://127.0.0.1:3000/auth/login -H \"Content-Type: application/json\" -d '{{\"username\":\"demo\",\"password\":\"password\"}}'"
    );

    axum::serve(listener, app)
        .await
        .map_err(|e| AuthError::internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn create_auth_framework() -> Result<Arc<AuthFramework>, Box<dyn std::error::Error>> {
    let database_url = get_dsn()?;
    let auth = AuthFramework::for_use_case(UseCasePreset::WebApp)
        .with_jwt()
        .secret_from_env("AUTH_JWT_SECRET_KEY")
        .done()
        .with_storage()
        .postgres(database_url)
        .done()
        .with_rate_limiting()
        .per_ip(requests(100).per_minute())
        .done()
        .security_preset(SecurityPreset::Development)
        .build()
        .await?;

    Ok(Arc::new(auth))
}

async fn create_app(auth: Arc<AuthFramework>) -> AuthFrameworkResult<Router> {
    let protected_routes = Router::new()
        .route("/auth/profile", get(protected_content_handler)) // if you have this
        .route("/protected", get(protected_content_handler))
        .route("/admin", get(admin_only_handler))
        .route(
            "/api/users",
            get(list_users_handler).post(create_user_handler),
        )
        .route(
            "/api/settings",
            get(get_settings_handler).post(update_settings_handler),
        )
        .layer(axum::middleware::from_fn_with_state(
            auth.clone(),
            simple_auth_middleware,
        ));
    let app = Router::new()
        .route("/", get(welcome_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/logout", post(logout_handler))
        .merge(protected_routes)
        .with_state(auth);

    Ok(app)
}

/// Public welcome handler
async fn welcome_handler() -> impl IntoResponse {
    Json(json!({
        "message": "Welcome to Auth Framework Axum Integration Demo!",
        "endpoints": {
            "login": "POST /auth/login",
            "profile": "GET /auth/profile (requires auth)",
            "protected": "GET /protected (requires auth)",
            "admin": "GET /admin (requires admin role)"
        },
        "example_login": {
            "url": "/auth/login",
            "method": "POST",
            "body": {
                "username": "demo",
                "password": "password"
            }
        }
    }))
}

/// Protected content handler - demonstrates automatic user extraction
async fn protected_content_handler() -> impl IntoResponse {
    Json(json!({
        "message": "This is protected content!",
        "user": {
            "id": "authenticated_user",
            "permissions": ["read", "write"],
            "roles": ["user"]
        },
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Admin-only handler - demonstrates role-based access
async fn admin_only_handler() -> impl IntoResponse {
    // In a real application, you'd check roles here or use middleware
    Json(json!({
        "message": "Welcome to the admin panel!",
        "admin_capabilities": [
            "user_management",
            "system_configuration",
            "audit_logs",
            "security_settings"
        ]
    }))
}

/// Simple protected handler that works with basic middleware
async fn simple_protected_handler() -> impl IntoResponse {
    Json(json!({
        "message": "This is protected content!",
        "note": "Authentication verified by middleware"
    }))
}

/// Simple admin handler that works with basic middleware
async fn simple_admin_handler() -> impl IntoResponse {
    Json(json!({
        "message": "Welcome to the admin panel!",
        "note": "Admin access verified"
    }))
}

/// Simple auth middleware for protected routes
async fn simple_auth_middleware(
    State(auth): State<Arc<AuthFramework>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extract Authorization header
    let headers = request.headers();
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .unwrap_or("");

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        // Validate token using AuthFramework's token manager
        if auth.token_manager().validate_jwt_token(token).is_ok() {
            return Ok(next.run(request).await);
        }
    }

    // Return unauthorized if no valid token
    Err(StatusCode::UNAUTHORIZED)
}

/// Simple login handler
async fn login_handler(
    State(_auth): State<Arc<AuthFramework>>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    // In a real app, validate credentials and create token
    Json(json!({
        "message": "Login endpoint",
        "token": "dummy-jwt-token",
        "user": payload.get("username").unwrap_or(&json!("unknown"))
    }))
}

/// Simple logout handler
async fn logout_handler() -> impl IntoResponse {
    Json(json!({
        "message": "Logged out successfully"
    }))
}

/// Demonstrate the protected route wrapper (alternative approach)
#[allow(dead_code)]
async fn create_app_with_protected_wrapper(
    auth: Arc<AuthFramework>,
) -> AuthFrameworkResult<Router> {
    // This is a placeholder for future enhanced integration
    // For now, we'll use the simplified approach
    let app = Router::new()
        .route("/", get(welcome_handler))
        .route("/protected", get(simple_protected_handler))
        .route("/admin", get(simple_admin_handler))
        .with_state(auth);

    Ok(app)
}

/// Example of custom middleware configuration
#[allow(dead_code)]
async fn create_app_with_custom_middleware(
    auth: Arc<AuthFramework>,
) -> AuthFrameworkResult<Router> {
    // Simplified implementation for now
    let protected_routes = Router::new()
        .route("/api/users", get(simple_protected_handler))
        .route("/api/settings", get(simple_protected_handler));

    let admin_routes = Router::new()
        .route("/admin/users", post(simple_admin_handler))
        .route("/admin/settings", post(simple_admin_handler));

    let app = Router::new()
        .route("/", get(welcome_handler))
        .merge(protected_routes)
        .merge(admin_routes)
        .with_state(auth);

    Ok(app)
}

// Additional handler examples
async fn list_users_handler() -> impl IntoResponse {
    Json(json!({"users": ["alice", "bob", "charlie"]}))
}

async fn get_settings_handler() -> impl IntoResponse {
    Json(json!({"theme": "dark", "notifications": true}))
}

async fn create_user_handler() -> impl IntoResponse {
    Json(json!({"message": "User created successfully"}))
}

async fn update_settings_handler() -> impl IntoResponse {
    Json(json!({"message": "Settings updated successfully"}))
}
