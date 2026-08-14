// Vigilant
// Database module
pub mod models;
pub mod queries;

use sqlx::postgres::PgPoolOptions;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{PgPool, SqlitePool};

/// A database pool that is either SQLite or Postgres, selected at runtime from
/// the `DATABASE_URL` scheme (`postgres://` / `postgresql://` → Postgres, else SQLite).
#[derive(Clone)]
pub enum DbPool {
    Sqlite(SqlitePool),
    Postgres(PgPool),
}

impl DbPool {
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await?;
            run_migrations_postgres(&pool).await?;
            Ok(Self::Postgres(pool))
        } else {
            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect(database_url)
                .await?;
            run_migrations_sqlite(&pool).await?;
            Ok(Self::Sqlite(pool))
        }
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, Self::Postgres(_))
    }

    pub fn as_sqlite(&self) -> &SqlitePool {
        match self {
            Self::Sqlite(p) => p,
            Self::Postgres(_) => panic!("expected SQLite pool"),
        }
    }

    pub fn as_postgres(&self) -> &PgPool {
        match self {
            Self::Postgres(p) => p,
            Self::Sqlite(_) => panic!("expected Postgres pool"),
        }
    }

    /// Readiness check — verifies the database is reachable. Returns `Ok(())`
    /// when a trivial query succeeds, regardless of dialect.
    pub async fn ping(&self) -> Result<(), sqlx::Error> {
        match self {
            Self::Sqlite(p) => sqlx::query("SELECT 1").execute(p).await.map(|_| ()),
            Self::Postgres(p) => sqlx::query("SELECT 1").execute(p).await.map(|_| ()),
        }
    }
}

pub async fn init_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    DbPool::connect(database_url).await
}

async fn run_migrations_sqlite(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let migrations = [
        include_str!("../../migrations/sqlite/001_initial.sql"),
        include_str!("../../migrations/sqlite/002_users_argon2.sql"),
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

async fn run_migrations_postgres(pool: &PgPool) -> Result<(), sqlx::Error> {
    let migrations = [
        include_str!("../../migrations/postgres/001_initial.sql"),
        include_str!("../../migrations/postgres/002_users_argon2.sql"),
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
