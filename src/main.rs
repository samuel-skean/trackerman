use axum::{extract::Path, response::IntoResponse, routing::{get, post, put}, Router};
use tokio::net::TcpListener;
use uuid::Uuid;

#[tokio::main]
async fn main() {

    // The API deliberately doesn't expose the term event, because I feel like
    // that's a bit ambiguous. It might refer to a whole kind of event we're
    // tracking, or just one instance of it. In the context of this code, an
    // *event* is always an instance, while a *tracker* has multiple events. We
    // may reconsider exposing this.

    // NOTE: I'm using trailing /'s for all paths, for consistency and because
    // it's what some tools (*cough* Go *cough*) seem to prefer. Ideally I would
    // redirect anyone who got this wrong, but I don't know how to do that yet
    // with Axum.
    let app = Router::new()
        // Each tracker:
        .route_service("/trackers/{tracker_id}/", put(put_event))
        .route_service("/trackers/{tracker_id}/description/", get(get_tracker_description))
        .route_service("/trackers/{tracker_id}/list/", get(get_tracker_events_list))
        .route_service("/trackers/{tracker_id}/status/", get(get_tracker_status))
        .route_service("/trackers/{tracker_id}/start/", post(start_event))
        .route_service("/trackers/{tracker_id}/stop/", post(stop_event))
        .route_service("/trackers/{tracker_id}/stop_and_increment/", post(stop_and_increment_event))

        // Multiple trackers:
        .route_service("/trackers/descriptions/", get(get_all_tracker_descriptions))
        // Tags also serve as human-readable names for trackers.
        .route_service("/tracker_tags/{tag}/", get(get_trackers_ids_by_tag));

    let listener = TcpListener::bind("0.0.0.0:2010").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn get_tracker_description(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!("Getting description of {tracker_id}\n")
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
    format!("Getting the status of {tracker_id} (whether it's ongoing and its current counter value)\n")
}

#[axum::debug_handler]
async fn start_event(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!("Attempting to start event for tracker {tracker_id}\n\
             This would fail if the event were ongoing\n")
}

#[axum::debug_handler]
async fn stop_event(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!("Attempting to stop event for tracker {tracker_id}\n\
             This would fail if the event were ongoing\n\
             You could supply a new value for the counter here.\n")
}

#[axum::debug_handler]
async fn stop_and_increment_event(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!("Attempting to stop event for tracker {tracker_id}\n\
             This would fail if the event were ongoing\n\
             You may not supply a new value for the counter here, it will simply be incremented\n")
}

#[axum::debug_handler]
async fn get_all_tracker_descriptions() -> impl IntoResponse {
    "Get all tracker descriptions\n"
}

#[axum::debug_handler]
async fn get_trackers_ids_by_tag(Path(tag): Path<String>) -> impl IntoResponse {
    format!("Definitely getting a list of tracker ids with the tag: {tag}\n")
}