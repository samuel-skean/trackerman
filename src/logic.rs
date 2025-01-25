use sqlx::{query, PgPool};
use uuid::Uuid;

pub async fn tracker_description(
    pool: &PgPool,
    tracker_id: Uuid,
) -> Result<Option<String>, sqlx::error::Error> {
    let res = query!(
        "SELECT \"description\" FROM trackers
         WHERE tracker_id = $1",
        tracker_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(res.map(|r| r.description))
}

// Effendy: status, start, stop
