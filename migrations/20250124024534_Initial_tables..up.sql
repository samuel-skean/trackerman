-- Add up migration script here 

-- TODO: Why can't I, the owner of a database, change it's public schema? As of
-- now, I need to run `CREATE SCHEMA AUTHORIZATION <username>` before this
--  for it to work, even if that username is the owner of the database.

CREATE TABLE trackers
(
    -- NOT NULL should be the default for PRIMARY KEYS, but just to be safe:
    tracker_id UUID PRIMARY KEY DEFAULT gen_random_uuid () NOT NULL,
    -- description gets highlighted, and was apparently a keyword in MS SQL at one point:
    "description" TEXT NOT NULL
);

CREATE TABLE events
(
    tracker_id UUID REFERENCES trackers NOT NULL,

    -- Precision to the second (matches integers in sqlite):
    start_time TIMESTAMP (0) NOT NULL,
    end_time TIMESTAMP (0) NOT NULL,

    new_value BIGINT NOT NULL,
    -- TODO: Maybe each of these should be unique, not just their combination?
    -- Should we allow overlaps for the same tracker?
    UNIQUE (start_time, end_time)
);