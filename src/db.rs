use chrono::Datelike;
use diesel::prelude::*;
use directories_next::ProjectDirs;
use models::Task;
use std::path::PathBuf;

pub mod models;
pub mod schema;

embed_migrations!();

pub fn path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("Zeitfresser", "Zeitfresser", "Zeitfresser").unwrap();
    let config_dir = proj_dirs.config_dir();
    let db_path = config_dir.join("database.sqlite3");
    db_path
}

pub fn establish_connection() -> SqliteConnection {
    let path = path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let conn = SqliteConnection::establish(path.to_str().unwrap()).unwrap();

    embedded_migrations::run(&conn).expect("could not run database migrations");
    conn
}

pub fn remove() {
    let path = path();
    std::fs::remove_file(path).expect("Could not remove db");
}

pub fn get_tasks(filter: models::Filter) -> Vec<Task> {
    let conn = establish_connection();
    use schema::tasks::dsl::*;
    let mut query = tasks.into_boxed();

    match filter {
        models::Filter::All => {}
        models::Filter::Day(date) => {
            let start = date.and_hms(0, 0, 0).naive_utc();
            let end = date.and_hms(23, 59, 59).naive_utc();
            query = query.filter(started_at.between(start, end))
        }
        models::Filter::Week => {
            let mut start = chrono::offset::Local::now().date();
            start = start - chrono::Duration::days(start.weekday().num_days_from_monday().into());
            let end = start + chrono::Duration::days(6);

            query = query.filter(started_at.between(
                start.and_hms(0, 0, 0).naive_utc(),
                end.and_hms(23, 59, 59).naive_utc(),
            ))
        }
    }

    query
        .order_by(started_at)
        .load::<Task>(&conn)
        .expect("Error loading tasks")
}

pub fn add_task(name: &str) {
    let conn = establish_connection();
    use schema::tasks;
    diesel::insert_into(tasks::table)
        .values((
            tasks::title.eq(name.to_string()),
            tasks::started_at.eq(diesel::dsl::now),
        ))
        .execute(&conn)
        .expect("Insertion failed");
}

pub fn finish_all() {
    let conn = establish_connection();
    use schema::tasks::dsl::*;
    let target = tasks.filter(finished_at.is_null());
    diesel::update(target)
        .set(finished_at.eq(diesel::dsl::now))
        .execute(&conn)
        .expect("Failed to update all");
}
