mod domain_types;
mod logic;

use std::{str::FromStr as _, sync::Arc};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_extra::routing::RouterExt as _;
use logic::create_tracker;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};
use uuid::Uuid;

struct AppState {
    db_conn_pool: PgPool,
}

#[tokio::main]
// Consider anyhow for errors, see [this post](https://www.reddit.com/r/rust/comments/17neomp/comment/k7rhrss/) for a nice breakdown.
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tracing: Generally following advice from fasterthanlime, here's an
    // example:
    // https://fasterthanli.me/series/building-a-rust-service-with-nix/part-4#adding-tracing
    //
    // One difference is, I'm using tower_http::TraceLayer which is
    // "higher-level" than the tracing output he gets. Also, not sure how he was
    // getting super detailed tracing for other crates (like hyper) which I
    // don't believe he directly configured.

    // Fetches max tracing level from the environment. CONSIDER: Using
    // with_env_filter for better defaults, as mentioned here:
    // https://www.ianbull.com/posts/axum-rust-tracing#how-with_env_filter-works
    // Not doing that right now to match fasterthanlime's stuff better and
    // because it seems a bit more complicated, especially to add fallback to
    // it. Don't just blindly follow that tutorial though, try using
    // EnvFilter::builder to provide fallback in a type-safe way.
    let filter = tracing_subscriber::filter::Targets::from_str(
        std::env::var("RUST_LOG").as_deref().unwrap_or("info"),
    )
    .expect("RUST_LOG should be a valid tracing filter");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .finish()
        .with(filter)
        .init();

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
        // Multiple trackers:
        .route_with_tsr("/trackers/", get(get_all_trackers).post(post_tracker))
        // Each tracker:
        .route_with_tsr("/trackers/{tracker_id}/", get(get_tracker_events_list))
        .route_with_tsr("/trackers/{tracker_id}/status/", get(get_tracker_status))
        // Yes, I know they're verbs. Try and stop me. (It's just easier given
        // that stop_and_increment/ exists, and Github does it too.)
        .route_with_tsr("/trackers/{tracker_id}/start/", post(start_event))
        .route_with_tsr("/trackers/{tracker_id}/stop/", post(stop_event))
        .route_with_tsr(
            "/trackers/{tracker_id}/stop_and_increment/",
            post(stop_and_increment_event),
        )
        .with_state(Arc::new(AppState { db_conn_pool }))
        .layer(TraceLayer::new_for_http());

    // TODO: Configure this port with an environment var.
    let listener = TcpListener::bind("0.0.0.0:2010").await.unwrap();

    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize, Serialize)]
struct NewTracker {
    name: String,
}

#[axum::debug_handler]
async fn post_tracker(
    State(state): State<Arc<AppState>>,
    Json(tracker): Json<NewTracker>,
) -> Result<impl IntoResponse, StatusCode> {
    // This logic should really live in logic.rs, but then we have to create unique error types.
    let tracker_id = create_tracker(&state.db_conn_pool, tracker.name)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .id;
    Ok((
        StatusCode::CREATED,
        [(header::LOCATION, format!("{tracker_id}/"))],
    ))
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
async fn get_all_trackers() -> impl IntoResponse {
    // This should return a map (JSON object) from tracker names to their URLS.
    "Get all tracker names & URLs\n"
}
