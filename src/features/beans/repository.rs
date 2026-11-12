use sqlx::PgPool;

use super::model::{BeanDraft, encode_notes};
use crate::core::errors::{AppError, database};

#[derive(Debug, sqlx::FromRow)]
pub(super) struct BeanRow {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) origin: String,
    pub(super) roaster: String,
    pub(super) process: String,
    pub(super) elevation: String,
    pub(super) tasting_notes: String,
    pub(super) image_key: Option<String>,
}

pub(super) async fn list(
    pool: &PgPool,
    user_id: &str,
    before: Option<i64>,
    limit: i64,
) -> Result<Vec<BeanRow>, AppError> {
    sqlx::query_as(
        r#"
        SELECT id, name, origin, roaster, process, elevation, tasting_notes, image_key
        FROM beans
        WHERE user_id = $1
          AND ($2::bigint IS NULL OR id < $2)
        ORDER BY id DESC
        LIMIT $3
        "#,
    )
    .bind(user_id)
    .bind(before)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|err| database("listing coffees", err))
}

pub(super) async fn fetch(pool: &PgPool, user_id: &str, id: i64) -> Result<BeanRow, AppError> {
    sqlx::query_as(
        r#"
        SELECT id, name, origin, roaster, process, elevation, tasting_notes, image_key
        FROM beans
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| database("loading a coffee", err))?
    .ok_or(AppError::BeanNotFound { id })
}

pub(super) async fn find_by_image_key(
    pool: &PgPool,
    user_id: &str,
    image_key: &str,
) -> Result<Option<BeanRow>, AppError> {
    sqlx::query_as(
        r#"
        SELECT id, name, origin, roaster, process, elevation, tasting_notes, image_key
        FROM beans
        WHERE user_id = $1 AND image_key = $2
        ORDER BY id DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .bind(image_key)
    .fetch_optional(pool)
    .await
    .map_err(|err| database("loading a coffee by its image", err))
}

pub(super) async fn insert(
    pool: &PgPool,
    user_id: &str,
    draft: &BeanDraft,
    image_key: Option<&str>,
) -> Result<BeanRow, AppError> {
    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO beans (
            user_id, name, origin, roaster, process, elevation, tasting_notes, image_key
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id
        "#,
    )
    .bind(user_id)
    .bind(&draft.name)
    .bind(&draft.origin)
    .bind(&draft.roaster)
    .bind(&draft.process)
    .bind(&draft.elevation)
    .bind(encode_notes(&draft.tasting_notes))
    .bind(image_key)
    .fetch_one(pool)
    .await
    .map_err(|err| database("saving a coffee", err))?;

    fetch(pool, user_id, id).await
}

pub(super) async fn update(
    pool: &PgPool,
    user_id: &str,
    id: i64,
    draft: &BeanDraft,
) -> Result<BeanRow, AppError> {
    let result = sqlx::query(
        r#"
        UPDATE beans
        SET name = $1,
            origin = $2,
            roaster = $3,
            process = $4,
            elevation = $5,
            tasting_notes = $6,
            updated_at = now()
        WHERE id = $7 AND user_id = $8
        "#,
    )
    .bind(&draft.name)
    .bind(&draft.origin)
    .bind(&draft.roaster)
    .bind(&draft.process)
    .bind(&draft.elevation)
    .bind(encode_notes(&draft.tasting_notes))
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(|err| database("updating a coffee", err))?;

    if result.rows_affected() == 0 {
        return Err(AppError::BeanNotUpdated { id });
    }

    fetch(pool, user_id, id).await
}

pub(super) async fn delete(
    pool: &PgPool,
    user_id: &str,
    id: i64,
) -> Result<Option<String>, AppError> {
    let row = fetch(pool, user_id, id).await?;
    sqlx::query("DELETE FROM beans WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(|err| database("deleting a coffee", err))?;
    Ok(row.image_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    fn sample() -> BeanDraft {
        BeanDraft {
            name: "Kieni AA".to_owned(),
            origin: "Nyeri, Kenya".to_owned(),
            roaster: "Kieni Washing Station".to_owned(),
            process: "Washed".to_owned(),
            elevation: "1,750 m".to_owned(),
            tasting_notes: vec![
                "Blackcurrant".to_owned(),
                "grapefruit".to_owned(),
                "brown sugar".to_owned(),
            ],
        }
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn creates_lists_updates_and_deletes_for_the_owner(pool: PgPool) {
        let created = insert(&pool, "mock-user", &sample(), Some("uploads/a.jpg"))
            .await
            .unwrap();
        assert_eq!(created.name, "Kieni AA");
        assert_eq!(created.image_key.as_deref(), Some("uploads/a.jpg"));
        assert_eq!(
            find_by_image_key(&pool, "mock-user", "uploads/a.jpg")
                .await
                .unwrap()
                .unwrap()
                .id,
            created.id
        );
        assert!(
            find_by_image_key(&pool, "someone-else", "uploads/a.jpg")
                .await
                .unwrap()
                .is_none()
        );

        assert_eq!(list(&pool, "mock-user", None, 100).await.unwrap().len(), 1);
        assert!(
            list(&pool, "someone-else", None, 100)
                .await
                .unwrap()
                .is_empty()
        );

        let mut edited = sample();
        edited.name = "Kieni AB".to_owned();
        let updated = update(&pool, "mock-user", created.id, &edited)
            .await
            .unwrap();
        assert_eq!(updated.name, "Kieni AB");
        assert_eq!(updated.image_key.as_deref(), Some("uploads/a.jpg"));
        assert!(matches!(
            update(&pool, "someone-else", created.id, &edited).await,
            Err(AppError::BeanNotUpdated { .. })
        ));

        let key = delete(&pool, "mock-user", created.id).await.unwrap();
        assert_eq!(key.as_deref(), Some("uploads/a.jpg"));
        assert!(matches!(
            fetch(&pool, "mock-user", created.id).await,
            Err(AppError::BeanNotFound { .. })
        ));
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn list_returns_a_page(pool: PgPool) {
        let older = insert(&pool, "mock-user", &sample(), None).await.unwrap();
        let mut second = sample();
        second.name = "Second".to_owned();
        let newer = insert(&pool, "mock-user", &second, None).await.unwrap();

        let page = list(&pool, "mock-user", None, 1).await.unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].id, newer.id);

        let rest = list(&pool, "mock-user", Some(newer.id), 1).await.unwrap();
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].id, older.id);
    }
}
