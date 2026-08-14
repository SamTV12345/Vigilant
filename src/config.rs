// Vigilant
// Minimal startup config — env vars + CLI

pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub listen_addr: String,
    pub assets_path: String,
}

impl AppConfig {
    pub fn load() -> Self {
        // Load .env file if present (ignore errors)
        let _ = dotenvy::dotenv();

        AppConfig {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:vigilant.db?mode=rwc".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
            listen_addr: resolve_listen_addr(),
            assets_path: std::env::var("ASSETS_PATH")
                .unwrap_or_else(|_| "./res/assets/".to_string()),
        }
    }
}

/// Resolve the listen address from `LISTEN_ADDR` (full `host:port`), with `PORT`
/// overriding only the port portion. This lets deployments bake a default
/// `LISTEN_ADDR` into the image while still changing the port via `PORT`
/// (the Kubernetes convention). Defaults to `0.0.0.0:8080`.
fn resolve_listen_addr() -> String {
    let mut addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());

    if let Ok(port) = std::env::var("PORT") {
        let port = port.trim();
        if !port.is_empty() {
            // Keep the host portion (handles IPv6 `[::1]:8080`) and swap the port.
            addr = match addr.rsplit_once(':') {
                Some((host, _)) => format!("{host}:{port}"),
                None => format!("0.0.0.0:{port}"),
            };
        }
    }

    addr
}
