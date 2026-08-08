// Vigilant
// Library entry point — enables integration tests in tests/
pub mod api;
pub mod config;
pub mod db;
pub mod notifier;
pub mod prober;

use std::sync::Arc;

pub use config::AppConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: Arc<AppConfig>,
    pub io: socketioxide::SocketIo,
}
