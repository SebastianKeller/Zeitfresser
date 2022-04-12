use chrono::prelude::*;

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: usize,
    pub start: DateTime<Utc>,
    pub end: Option<DateTime<Utc>>,
    pub name: String,
}

impl Entry {
    pub fn new(name: String) -> Entry {
        return Entry {
            id: 0,
            start: Utc::now(),
            end: None,
            name,
        };
    }
}
