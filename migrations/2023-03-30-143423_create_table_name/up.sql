-- Your SQL goes here
CREATE TABLE blogs (
    id INTEGER NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    posted_at TIMESTAMP NOT NULL,
    jp_tweeted BOOLEAN NOT NULL DEFAULT FALSE,
    eng_tweeted BOOLEAN NOT NULL DEFAUlT FALSE
);