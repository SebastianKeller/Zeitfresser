use serde::Serialize;

#[derive(Debug, Serialize, Queryable, Clone)]
pub struct Task {
    pub id: i32,
    pub title: String,
    pub started_at: chrono::NaiveDateTime,
    pub finished_at: Option<chrono::NaiveDateTime>,
}

impl Task {
    pub fn duration(&self) -> chrono::Duration {
        let finished_at = self.finished_at.unwrap_or(chrono::Utc::now().naive_utc());
        finished_at - self.started_at
    }
}

#[derive(Debug)]
pub enum Filter {
    All,
    Day(chrono::Date<chrono::Local>),
    Week,
    Last,
}
