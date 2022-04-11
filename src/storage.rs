use crate::data::Entry;
use chrono::prelude::*;
use rusqlite::params;
use std::path::Path;

pub trait Storage {
    fn entries(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Entry>, rusqlite::Error>;
    fn current_entries(&self) -> Result<Vec<Entry>, rusqlite::Error>;
    fn entrie_by_id(&self, id: usize) -> Result<Entry, rusqlite::Error>;
    fn add_entry(&self, entry: Entry);
    fn update_entry(&self, entry: Entry) -> Result<(), rusqlite::Error>;
}

pub struct SqlStorage {
    connection: rusqlite::Connection,
}

impl SqlStorage {
    pub fn file<P: AsRef<Path>>(path: P) -> SqlStorage {
        let connection = rusqlite::Connection::open(path).unwrap();
        SqlStorage::init_db(&connection);
        return SqlStorage { connection };
    }

    fn init_db(connection: &rusqlite::Connection) {
        connection
        .execute(
            "CREATE TABLE IF NOT EXISTS entries (id INTEGER PRIMARY KEY AUTOINCREMENT, start DATETIME, end DATETIME, name STRING NOT NULL)",
            [],
        )
        .unwrap();
    }
}

impl Storage for SqlStorage {
    fn add_entry(&self, entry: Entry) {
        self.connection
            .execute(
                "INSERT INTO entries (start, end, name) VALUES(?, ?, ?)",
                params![&entry.start, &entry.end, &entry.name],
            )
            .unwrap_or_else(|error| panic!("Could not insert entry! {:?}", error));

        let last_row = self.connection.last_insert_rowid() as usize;
        if let Ok(mut prev) = self.entrie_by_id(last_row - 1) {
            if prev.end == None {
                prev.end = Some(entry.start);
            }

            match self.update_entry(prev) {
                Err(e) => println!("{}", e),
                Ok(()) => (),
            }

            return;
        }
    }

    fn current_entries(&self) -> Result<Vec<Entry>, rusqlite::Error> {
        let start = Local::today().and_hms(0, 0, 0).with_timezone(&Utc);
        let end = Local::today().succ().and_hms(0, 0, 0).with_timezone(&Utc);
        return self.entries(start, end);
    }

    fn entrie_by_id(&self, id: usize) -> Result<Entry, rusqlite::Error> {
        self.connection.query_row(
            "SELECT id, start, end, name FROM entries where id=?",
            [id],
            |row| {
                Ok(Entry {
                    id: row.get(0)?,
                    start: row.get(1)?,
                    end: row.get(2)?,
                    name: row.get(3)?,
                })
            },
        )
    }

    fn entries(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<Entry>, rusqlite::Error> {
        let mut stmt = self.connection.prepare(
            "SELECT id, start, end, name FROM entries where start between datetime(?) and datetime(?)",
        )?;

        let iter = stmt.query_map([&from, &to], |row| {
            Ok(Entry {
                id: row.get(0)?,
                start: row.get(1)?,
                end: row.get(2)?,
                name: row.get(3)?,
            })
        })?;

        let mut entries = Vec::new();
        for i in iter {
            entries.push(i?);
        }
        Ok(entries)
    }

    fn update_entry(&self, entry: Entry) -> Result<(), rusqlite::Error> {
        match self.connection.execute(
            "UPDATE entries set start = ?, end = ?, name = ? where id == ?",
            params![&entry.start, &entry.end, &entry.name, &entry.id],
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl SqlStorage {
        pub fn memory() -> SqlStorage {
            let connection = rusqlite::Connection::open_in_memory().unwrap();
            SqlStorage::init_db(&connection);
            return SqlStorage { connection };
        }
    }

    #[test]
    fn add_entry() {
        let storage = SqlStorage::memory();
        storage.add_entry(Entry {
            id: 0,
            start: Utc::now(),
            end: Some(Utc::now()),
            name: "test".to_string(),
        });
        assert_eq!(storage.current_entries().unwrap().len(), 1);
    }
}
