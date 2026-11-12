use std::time::Duration;

use sqlx::PgPool;

use super::ClaimedUpload;
use crate::core::errors::{AppError, database};

pub(super) async fn insert(pool: &PgPool, user_id: &str, key: &str) -> Result<(), AppError> {
    sqlx::query("INSERT INTO uploads (key, user_id) VALUES ($1, $2)")
        .bind(key)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|err| database("saving an upload ticket", err))?;
    Ok(())
}

pub(super) async fn claim(
    pool: &PgPool,
    user_id: &str,
    key: &str,
) -> Result<ClaimedUpload, AppError> {
    let row: Option<String> = sqlx::query_scalar(
        r#"
        UPDATE uploads
        SET in_processing = true
        WHERE key = $1 AND user_id = $2 AND in_processing = false
        RETURNING key
        "#,
    )
    .bind(key)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| database("claiming an upload ticket", err))?;

    if let Some(key) = row {
        return Ok(ClaimedUpload { key });
    }

    let in_processing: Option<bool> =
        sqlx::query_scalar("SELECT in_processing FROM uploads WHERE key = $1 AND user_id = $2")
            .bind(key)
            .bind(user_id)
            .fetch_optional(pool)
            .await
            .map_err(|err| database("checking an upload ticket", err))?;

    match in_processing {
        Some(_) => Err(AppError::UploadInProgress),
        None => Err(AppError::UploadNotFound),
    }
}

pub(super) async fn release_processing(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query("UPDATE uploads SET in_processing = false WHERE in_processing")
        .execute(pool)
        .await
        .map_err(|err| database("releasing in-progress upload tickets", err))?;
    Ok(())
}

pub(super) async fn release(pool: &PgPool, key: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE uploads SET in_processing = false WHERE key = $1")
        .bind(key)
        .execute(pool)
        .await
        .map_err(|err| database("releasing an upload ticket", err))?;
    Ok(())
}

pub(super) async fn delete(pool: &PgPool, key: &str) -> Result<(), AppError> {
    sqlx::query("DELETE FROM uploads WHERE key = $1")
        .bind(key)
        .execute(pool)
        .await
        .map_err(|err| database("deleting an upload ticket", err))?;
    Ok(())
}

pub(super) async fn delete_abandoned(
    pool: &PgPool,
    abandon_after: Duration,
) -> Result<Vec<String>, AppError> {
    sqlx::query_scalar(
        r#"
        DELETE FROM uploads
        WHERE in_processing = false
          AND created_at < now() - ($1::bigint * INTERVAL '1 second')
          AND NOT EXISTS (
              SELECT 1 FROM beans WHERE beans.image_key = uploads.key
          )
        RETURNING key
        "#,
    )
    .bind(abandon_after.as_secs() as i64)
    .fetch_all(pool)
    .await
    .map_err(|err| database("expiring abandoned uploads", err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./migrations")]
    async fn claim_is_once_per_owner_and_can_be_restored(pool: PgPool) {
        insert(&pool, "mock-user", "uploads/a.jpg").await.unwrap();

        assert!(matches!(
            claim(&pool, "someone-else", "uploads/a.jpg").await,
            Err(AppError::UploadNotFound)
        ));

        let claimed = claim(&pool, "mock-user", "uploads/a.jpg").await.unwrap();
        assert_eq!(claimed.key, "uploads/a.jpg");
        assert!(matches!(
            claim(&pool, "mock-user", "uploads/a.jpg").await,
            Err(AppError::UploadInProgress)
        ));

        release(&pool, &claimed.key).await.unwrap();
        assert_eq!(
            claim(&pool, "mock-user", "uploads/a.jpg")
                .await
                .unwrap()
                .key,
            "uploads/a.jpg"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn release_processing_clears_in_flight_tickets(pool: PgPool) {
        insert(&pool, "mock-user", "uploads/a.jpg").await.unwrap();
        claim(&pool, "mock-user", "uploads/a.jpg").await.unwrap();

        release_processing(&pool).await.unwrap();

        assert_eq!(
            claim(&pool, "mock-user", "uploads/a.jpg")
                .await
                .unwrap()
                .key,
            "uploads/a.jpg"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn delete_abandoned_keeps_live_and_referenced_tickets(pool: PgPool) {
        insert(&pool, "mock-user", "old").await.unwrap();
        insert(&pool, "mock-user", "processing").await.unwrap();
        insert(&pool, "mock-user", "fresh").await.unwrap();
        insert(&pool, "mock-user", "referenced").await.unwrap();

        sqlx::query(
            "UPDATE uploads SET created_at = '2000-01-01 00:00:00+00' WHERE key != 'fresh'",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("UPDATE uploads SET in_processing = true WHERE key = 'processing'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query(
            "INSERT INTO beans (user_id, name, origin, roaster, process, elevation, tasting_notes, image_key) VALUES ('mock-user', 'A', '', '', '', '', '[]', 'referenced')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let gone: std::collections::HashSet<String> =
            delete_abandoned(&pool, Duration::from_secs(60))
                .await
                .unwrap()
                .into_iter()
                .collect();
        assert_eq!(gone, std::collections::HashSet::from(["old".to_owned()]));

        let left: Vec<String> = sqlx::query_scalar("SELECT key FROM uploads ORDER BY key")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(
            left,
            vec![
                "fresh".to_owned(),
                "processing".to_owned(),
                "referenced".to_owned()
            ]
        );
    }
}
