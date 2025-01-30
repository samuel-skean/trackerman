use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub struct Tracker {
    pub id: Uuid,
    pub human_name: String,
}
