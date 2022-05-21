use serde::Serialize;

#[derive(Debug, Serialize, Queryable)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub started_at: chrono::NaiveDateTime,
    pub finished_at: Option<chrono::NaiveDateTime>,
}
