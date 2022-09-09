// @generated automatically by Diesel CLI.

diesel::table! {
    sources (id) {
        id -> Integer,
        subreddit -> Nullable<Text>,
    }
}
