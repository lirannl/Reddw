// @generated automatically by Diesel CLI.

diesel::table! {
    config (id) {
        id -> Nullable<Integer>,
        refresh_minutes -> Nullable<Integer>,
        allow_nsfw -> Nullable<Integer>,
    }
}

diesel::table! {
    sources (id) {
        id -> Integer,
        subreddit -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    config,
    sources,
);
