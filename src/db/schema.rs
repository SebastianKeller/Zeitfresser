table! {
    tasks (id) {
        id -> Integer,
        title -> Text,
        started_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
    }
}
