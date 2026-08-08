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
            listen_addr: std::env::var("LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            assets_path: std::env::var("ASSETS_PATH")
                .unwrap_or_else(|_| "./res/assets/".to_string()),
        }
    }
}
