use sqlx::{query_as, PgPool};
use tracing::error;
use uuid::Uuid;

use crate::domain_types::{Event, Tracker};

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

pub async fn tracker_events(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Vec<Event>>, sqlx::error::Error> {
    // This is a lot of type shenanigans :(.

    struct NullableEvent {
        tracker_id: Option<Uuid>,
        start_time: Option<chrono::NaiveDateTime>,
        end_time: Option<chrono::NaiveDateTime>,
        new_value: Option<i64>,
    }
    impl TryFrom<NullableEvent> for Event {
        type Error = ();

        fn try_from(value: NullableEvent) -> Result<Self, Self::Error> {
            match value {
                NullableEvent {
                    // The main thing I care about is the tracker_id, since the
                    // event's tracker_id can only be null if there is no such
                    // tracker since it must reference a tracker, but I might as
                    // well check the nullability of the rest of these things
                    // anyway.
                    tracker_id: None,
                    start_time: None,
                    end_time: None,
                    new_value: None,
                } => Err(()),
                NullableEvent {
                    tracker_id: Some(_),
                    start_time: Some(start_time),
                    end_time,
                    new_value: Some(new_value),
                } => Ok(Self {
                    start_time,
                    end_time,
                    new_value,
                }),
                _ => {
                    error!("Inconsistent nullable event. Possible schema error?");
                    unreachable!()
                }
            }
        }
    }

    // The sqlx "casts" to nullable fields, which use double-quotes,
    // (https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns)
    // are used to counteract [this
    // issue](https://github.com/launchbadge/sqlx/issues/2127) regarding
    // inferring the nullability of the results of some asymmetric joins. They
    // should be removable once the fix is merged. From the sqlx documentation
    // linked above, it seems this is not an issue with MySQL.
    let empty_if_no_tracker: Option<Vec<Event>> = query_as!(
        NullableEvent,
        "SELECT events.tracker_id as \"tracker_id?\", events.start_time as \"start_time?\",
           events.end_time as \"end_time?\", events.new_value as \"new_value?\"
           FROM events RIGHT OUTER JOIN trackers ON (events.tracker_id = trackers.id)
           WHERE trackers.id = $1",
        id
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|ev| ev.try_into().ok())
    .collect();

    Ok(match empty_if_no_tracker {
        Some(v) if v.is_empty() => None,
        Some(v) => Some(v),
        None => Some(vec![]),
    })
}

// TODO: Effendy: status, start, stop
