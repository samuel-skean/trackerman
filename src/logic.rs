use sqlx::{query_as, PgPool};

use crate::domain_types::Tracker;

pub async fn create_tracker(
    pool: &PgPool,
    human_name: String,
) -> Result<Tracker, sqlx::error::Error> {
    query_as!(
        Tracker,
        "INSERT INTO trackers (human_name) VALUES ($1) RETURNING *",
        human_name
    )
    .fetch_one(pool)
    .await
}

// TODO: Effendy: status, start, stop