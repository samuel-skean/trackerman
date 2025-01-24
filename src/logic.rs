use sqlx::{query, PgPool};
use uuid::Uuid;

pub async fn tracker_description(pool: &PgPool, tracker_id: Uuid) -> Result<String, Box<dyn std::error::Error>> {
   let res = 
      query!(
         "SELECT \"description\" FROM trackers
         WHERE tracker_id = $1",
         tracker_id
      )
      .fetch_one(pool).await?;
   Ok(res.description)
}