use std::path::Path;

use chrono::Datelike;
use diesel::prelude::*;
use directories_next::ProjectDirs;

pub mod models;
pub mod schema;

pub use models::Filter;
pub use models::Task;

embed_migrations!();

pub struct DB {
    connection: SqliteConnection,
}

impl DB {
    pub fn new_xdg() -> DB {
        let proj_dirs = ProjectDirs::from("Zeitfresser", "Zeitfresser", "Zeitfresser").unwrap();
        let config_dir = proj_dirs.config_dir();
        let db_path = config_dir.join("database.sqlite3");
        std::fs::create_dir_all(config_dir).unwrap();
        DB::new(&db_path)
    }

    pub fn new(path: &Path) -> DB {
        let conn = SqliteConnection::establish(path.to_str().unwrap()).unwrap();
        embedded_migrations::run(&conn).expect("could not run database migrations");
        DB { connection: conn }
    }

    pub fn get_tasks(&self, filter: models::Filter) -> Vec<Task> {
        use schema::tasks::dsl::*;
        let mut query = tasks.into_boxed();

        match filter {
            models::Filter::Last => query = query.order_by(started_at.desc()).limit(1),
            _ => query = query.order_by(started_at.asc()),
        }

        match filter {
            models::Filter::All => {}
            models::Filter::Last => {}
            models::Filter::Day(date) => {
                let start = date.and_hms(0, 0, 0).naive_utc();
                let end = date.and_hms(23, 59, 59).naive_utc();
                query = query.filter(started_at.between(start, end))
            }
            models::Filter::Week => {
                let mut start = chrono::offset::Local::now().date();
                start =
                    start - chrono::Duration::days(start.weekday().num_days_from_monday().into());
                let end = start + chrono::Duration::days(6);

                query = query.filter(started_at.between(
                    start.and_hms(0, 0, 0).naive_utc(),
                    end.and_hms(23, 59, 59).naive_utc(),
                ))
            }
        }

        query
            .load::<Task>(&self.connection)
            .expect("Error loading tasks")
    }

    pub fn add_task(&self, name: &str) {
        use schema::tasks;
        diesel::insert_into(tasks::table)
            .values((
                tasks::title.eq(name.to_string()),
                tasks::started_at.eq(diesel::dsl::now),
            ))
            .execute(&self.connection)
            .expect("Insertion failed");
    }

    pub fn finish_all(&self) {
        use schema::tasks::dsl::*;
        let target = tasks.filter(finished_at.is_null());
        diesel::update(target)
            .set(finished_at.eq(diesel::dsl::now))
            .execute(&self.connection)
            .expect("Failed to update all");
    }

    pub fn update_task(&self, task: Task) {
        use schema::tasks::dsl::*;
        diesel::update(tasks.filter(id.eq(task.id)))
            .set((
                finished_at.eq(task.finished_at),
                title.eq(task.title)
            ))
            .execute(&self.connection)
            .expect("Failed to update task");
    }

    pub fn clear_tasks(&self) {
        use schema::tasks;
        diesel::delete(tasks::table)
            .execute(&self.connection)
            .expect("Failed to delete rows from tasks table");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn db() -> DB {
        let conn = SqliteConnection::establish(":memory:").unwrap();
        embedded_migrations::run(&conn).expect("could not run database migrations");
        DB { connection: conn }
    }

    #[test]
    fn add_task() {
        let db = db();
        let names = vec!["a", "b", "c"];
        for n in &names {
            db.add_task(n);
        }

        let tasks = db.get_tasks(Filter::All);
        assert_eq!(3, tasks.len());
    }

    #[test]
    fn finish_all() {
        let db = db();
        let names = vec!["a", "b", "c"];
        for n in names {
            db.add_task(n);
        }
        db.finish_all();
        let tasks = db.get_tasks(Filter::All);
        for t in &tasks {
            assert!(t.finished_at != None)
        }
    }

    #[test]
    fn clear_tasks() {
        let db = db();
        db.add_task("a");
        db.clear_tasks();
        let tasks = db.get_tasks(Filter::All);
        assert_eq!(0, tasks.len());
    }
}
