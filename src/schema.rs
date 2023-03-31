// @generated automatically by Diesel CLI.

diesel::table! {
    blogs (id) {
        id -> Integer,
        name -> Text,
        posted_at -> Timestamp,
        jp_tweeted -> Bool,
        eng_tweeted -> Bool,
    }
}

diesel::table! {
    news (id) {
        id -> Integer,
        news_id -> Text,
        posted_at -> Timestamp,
        jp_tweeted -> Bool,
        eng_tweeted -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    blogs,
    news,
);
