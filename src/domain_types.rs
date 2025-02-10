use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct Tracker {
    pub id: Uuid,
    pub human_name: String,
}

#[derive(Deserialize, Serialize, Debug)]

// question: shouldn't this struct contain the tracker_id?
pub struct Event {
    pub start_time: chrono::NaiveDateTime,
    pub end_time: Option<chrono::NaiveDateTime>,
    pub new_value: i64,
}
