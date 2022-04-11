mod data;
mod storage;

use directories_next::ProjectDirs;
use storage::{Storage, SqlStorage};

fn main() {
    let proj_dirs = ProjectDirs::from("Zeitfresser", "Zeitfresser", "Zeitfresser").unwrap();
    let config_dir = proj_dirs.config_dir();
    let db_path = config_dir.join("database.sqlite3");
    std::fs::create_dir_all(&config_dir).unwrap();

    let storage = SqlStorage::file(db_path);
    let e = data::Entry::new("proggen".to_string());
    storage.add_entry(e);

    for entry in storage.current_entries().unwrap() {
        println!("{:?}", entry);
    }
}
