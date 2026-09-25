use std::str::FromStr;

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};

use super::config::DatabaseConfig;

pub(crate) async fn connect(config: &DatabaseConfig) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(&config.url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(config.busy_timeout);

    SqlitePoolOptions::new()
        .max_connections(config.pool_size)
        .connect_with(options)
        .await
}

pub(crate) async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

#[cfg(test)]
pub(crate) async fn test_db() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .unwrap();
    migrate(&pool).await.unwrap();
    pool
}
