use std::path::PathBuf;

use diesel::prelude::*;
pub mod models;
pub mod schema;

use directories_next::ProjectDirs;
use models::Task;

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

pub fn get_tasks() -> Vec<Task> {
    let conn = establish_connection();
    use schema::tasks::dsl::*;
    tasks
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

pub fn end_all() {
    let conn = establish_connection();
    use schema::tasks::dsl::*;
    let target = tasks.filter(finished_at.is_null());
    diesel::update(target)
        .set(finished_at.eq(diesel::dsl::now))
        .execute(&conn)
        .expect("Failed to update all");
}
