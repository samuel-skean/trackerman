use axum::{extract::Path, response::IntoResponse, routing::{get, post, put}, Router};
use tokio::net::TcpListener;
use uuid::Uuid;

#[tokio::main]
async fn main() {

    // TODO: Even I feel like this API is verbose, and I like Java! It also
    // deliberately doesn't expose the term event, because I feel like that's a
    // bit ambiguous. It might refer to a whole kind of event we're tracking, or
    // just one instance of it. In the context of this code, an *event* is
    // always an instance, while a *tracker* has multiple events. We may
    // reconsider exposing this.

    // NOTE: I'm using trailing /'s for all paths, for consistency and because
    // it's what some tools (*cough* Go *cough*) seem to prefer. Ideally I would
    // redirect anyone who got this wrong, but I don't know how to do that yet
    // with Axum.
    let app = Router::new()
        // Each tracker:
        .route_service("/trackers/{tracker_id}/", put(put_event))
        .route_service("/trackers/{tracker_id}/summary/", get(get_tracker_summary))
        .route_service("/trackers/{tracker_id}/list/", get(get_tracker_events_list))
        .route_service("/trackers/{tracker_id}/status/", get(get_tracker_status))
        .route_service("/trackers/{tracker_id}/start/", post(start_event))
        .route_service("/trackers/{tracker_id}/stop/", post(stop_event))
        .route_service("/trackers/{tracker_id}/stop_and_increment/", post(stop_and_increment_event))

        // Multiple trackers:
        .route_service("/trackers/summaries/", get(get_all_tracker_summaries))
        .route_service("/trackers/lists/", get(get_all_tracker_event_lists))
        // Tags also serve as human-readable names for trackers.
        .route_service("/tracker_tags/{tag}/summaries/", get(get_tracker_summaries_by_tag))
        .route_service("/tracker_tags/{tag}/list/", get(get_tracker_event_lists_by_tag));

    let listener = TcpListener::bind("0.0.0.0:2010").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[axum::debug_handler]
async fn get_tracker_summary(Path(tracker_id): Path<Uuid>) -> impl IntoResponse {
    format!("Getting summary of {tracker_id}\n")
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
async fn get_all_tracker_summaries() -> impl IntoResponse {
    "Get all tracker summaries\n"
}

#[axum::debug_handler]
async fn get_all_tracker_event_lists() -> impl IntoResponse {
    "Get all lists of all the events for all the trackers.\n\
     Is this even a good idea?\n"
}

#[axum::debug_handler]
async fn get_tracker_summaries_by_tag(Path(tag): Path<String>) -> impl IntoResponse {
    format!("Definitely getting tracker summaries with the tag: {tag}\n")
}

#[axum::debug_handler]
async fn get_tracker_event_lists_by_tag(Path(tag): Path<String>) -> impl IntoResponse {
    format!("Definitely getting tracker event lists with the tag: {tag}\n")
}