mod logic;

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use axum_extra::routing::RouterExt as _;
use logic::tracker_description;
use sqlx::PgPool;
use tokio::net::TcpListener;
use uuid::Uuid;

struct AppState {
    db_conn_pool: PgPool,
}

#[tokio::main]
// Consider anyhow for errors, see [this post](https://www.reddit.com/r/rust/comments/17neomp/comment/k7rhrss/) for a nice breakdown.
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_conn_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL env var must be set. TODO: Support .env files in the running app.");

    let db_conn_pool = sqlx::PgPool::connect(&db_conn_url).await?;

    // The API deliberately doesn't expose the term event, because I feel like
    // that's a bit ambiguous. It might refer to a whole kind of event we're
    // tracking, or just one instance of it. In the context of this code, an
    // *event* is always an instance, while a *tracker* has multiple events. We
    // may reconsider exposing this.

    // NOTE: I'm redirecting all paths to paths that end in trailing slashes.
    // This seems to be the default preferred by Go. This seems significantly
    // better than normalizing paths internally, since the client is made aware
    // of the canonical URL here.
    //
    // It's kinda ugly that I have to do this for each endpoint - I'd rather
    // there were some middleware that did it that I could apply to the whole
    // router... but oh well.
    let app = Router::new()
        // Each tracker:
        .route_with_tsr("/trackers/{tracker_id}/", put(put_event))
        // TODO: Allow clients to set the description as well.
        .route_with_tsr(
            "/trackers/{tracker_id}/description/",
            get(get_tracker_description),
        )
        .route_with_tsr("/trackers/{tracker_id}/list/", get(get_tracker_events_list))
        .route_with_tsr("/trackers/{tracker_id}/status/", get(get_tracker_status))
        .route_with_tsr("/trackers/{tracker_id}/start/", post(start_event))
        .route_with_tsr("/trackers/{tracker_id}/stop/", post(stop_event))
        .route_with_tsr(
            "/trackers/{tracker_id}/stop_and_increment/",
            post(stop_and_increment_event),
        )
        // Multiple trackers:
        .route_with_tsr("/trackers/descriptions/", get(get_all_tracker_descriptions))
        // Tags also serve as human-readable names for trackers.
        .route_with_tsr("/tracker_tags/{tag}/", get(get_trackers_ids_by_tag))
        .with_state(Arc::new(AppState { db_conn_pool }));

    // TODO: Configure this port with an environment var.
    let listener = TcpListener::bind("0.0.0.0:2010").await.unwrap();

    axum::serve(listener, app).await?;
    Ok(())
}

#[axum::debug_handler]
async fn get_tracker_description(
    State(state): State<Arc<AppState>>,
    Path(tracker_id): Path<Uuid>,
) -> Result<String, StatusCode> {
    tracker_description(&state.db_conn_pool, tracker_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .transpose()
        .unwrap_or(Err(StatusCode::NOT_FOUND))
}

#[axum::debug_handler]
async fn put_event(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!("Putting an event at {tracker_id}\n")
}

#[axum::debug_handler]
async fn get_tracker_events_list(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!("Getting events list for {tracker_id}\n")
}

#[axum::debug_handler]
async fn get_tracker_status(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!(
        "Getting the status of {tracker_id} (whether it's ongoing and its current counter value)\n"
    )
}

#[axum::debug_handler]
async fn start_event(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!(
        "Attempting to start event for tracker {tracker_id}\n\
             This would fail if the event were ongoing\n"
    )
}

#[axum::debug_handler]
async fn stop_event(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!(
        "Attempting to stop event for tracker {tracker_id}\n\
             This would fail if the event were ongoing\n\
             You could supply a new value for the counter here.\n"
    )
}

#[axum::debug_handler]
async fn stop_and_increment_event(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!(
        "Attempting to stop event for tracker {tracker_id}\n\
             This would fail if the event were ongoing\n\
             You may not supply a new value for the counter here, it will simply be incremented\n"
    )
}

#[axum::debug_handler]
async fn get_all_tracker_descriptions() -> impl IntoResponse {
    "Get all tracker descriptions\n"
}

#[axum::debug_handler]
async fn get_trackers_ids_by_tag(Path(tag): Path<String>) -> impl IntoResponse {
    format!("Definitely getting a list of tracker ids with the tag: {tag}\n")
}
