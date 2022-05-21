-- Your SQL goes here
CREATE TABLE tasks (
    id INTEGER NOT NULL PRIMARY KEY,
    title VARCHAR NOT NULL,
    started_at DATETIME NOT NULL,
    finished_at DATETIME
)
