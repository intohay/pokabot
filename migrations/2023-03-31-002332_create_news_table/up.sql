-- Your SQL goes here
CREATE TABLE news (
    id INTEGER NOT NULL PRIMARY KEY,
    news_id TEXT NOT NULL,
    posted_at TIMESTAMP NOT NULL,
    jp_tweeted BOOLEAN NOT NULL DEFAULT FALSE,
    eng_tweeted BOOLEAN NOT NULL DEFAUlT FALSE
);