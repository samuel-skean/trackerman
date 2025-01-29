-- Add up migration script here 

CREATE TABLE trackers
(
    -- NOT NULL should be the default for PRIMARY KEYS, but just to be safe:
    tracker_id UUID PRIMARY KEY DEFAULT gen_random_uuid () NOT NULL,
    tracker_name TEXT UNIQUE NOT NULL
);

CREATE TABLE events
(
    tracker_id UUID REFERENCES trackers NOT NULL,

    -- Primary Key can only be start_time because as of now, end_time must be
    -- nullable to reflect an in-flight event. Consider a separate
    -- events_in_flight table with no end_time and more natively expressed
    -- constraints.
    
    -- Precision to the second (matches integers in sqlite):
    start_time TIMESTAMP (0) PRIMARY KEY NOT NULL CONSTRAINT ends_after_start CHECK (end_time >= start_time),
    end_time TIMESTAMP (0),

    new_value BIGINT NOT NULL
    -- TODO: Prohibit overlaps for the same tracker.
);