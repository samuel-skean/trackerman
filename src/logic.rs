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
           FROM trackers LEFT OUTER JOIN events ON (events.tracker_id = trackers.id)
           WHERE trackers.id = $1",
        id
    )
    // Some of the following code should probably be factored out into a
    // function as we write more code that manipulates NullableEvents. It also
    // may not be the best solution - it might be better to choose the less
    // magical path for `collect`, below, collecting into a
    // `Vec<Option<Event>>`. But I'm not sure.
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|ev| ev.try_into().ok())
    // This `collect` seems kind of magical, turning an iterator that produces
    // `Option<Events>` into a `Option<Vec<Event>>` instead of the more
    // straightforward translation, turning it into a `Vec<Option<Event>>`
    // (which also compiles). I assume this conversion produces None if any of
    // the items in the Iterator are `None`, and produces some vector otherwise,
    // but I haven't found the code that actually implements this.
    //
    // This is exactly what we want: we want to turn this per-row attribute of
    // some of the fields being null into an attribute of the full result set
    // (representing a tracker that exists, but has no events). In this case,
    // there's only ever *one* of these rows with some nulls, and if it exists
    // it's the only row, because we filtered on the tracker_id. We don't ensure
    // that set of conditions here, which is the only downside to the magic I
    // see. But I think this code is long enough. Unfortunately, there's still
    // more shenanigans...
    .collect();

    // ...Here are the shenanigans. In essence, we need to flip the case of None
    // and the empty list.
    Ok(match empty_if_no_tracker {
        // An empty result set means no such tracker exists:
        Some(v) if v.is_empty() => None,
        // A non-empty result set means a tracker exists and has events:
        Some(v) => Some(v),
        // A row with some nulls, which we turned into a None, means a tracker
        // exists but has no events.
        None => Some(vec![]),
    })
}

pub async fn all_trackers(pool: &PgPool) -> Result<Vec<Tracker>, sqlx::error::Error> {
    query_as!(Tracker, "SELECT * FROM trackers")
        .fetch_all(pool)
        .await
}

// TODO: Effendy: status, start, stop
