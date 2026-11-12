use sqlx::{PgPool, postgres::PgPoolOptions};

use super::config::DatabaseConfig;

pub(crate) async fn connect(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.pool_size)
        .connect(&config.url)
        .await
}

pub(crate) async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
