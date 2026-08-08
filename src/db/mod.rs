// Vigilant
// Database module
pub mod models;
pub mod queries;

use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

pub async fn init_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Run migrations from the embedded SQL files
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let migrations = [
        include_str!("../../migrations/001_initial.sql"),
        include_str!("../../migrations/002_users_argon2.sql"),
    ];

    for migration in migrations {
        for statement in migration.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                let _ = sqlx::query(trimmed).execute(pool).await;
            }
        }
    }

    Ok(())
}
