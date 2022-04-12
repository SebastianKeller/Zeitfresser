mod data;
mod storage;

use chrono::prelude::*;
use clap::{arg, Command};
use directories_next::ProjectDirs;
use storage::{SqlStorage, Storage};

fn cli() -> Command<'static> {
    Command::new("zeitfresser")
        .about("A tool to track where your time is spent.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Tracks a new task")
                .arg(arg!(<NAME> "The name of the task"))
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("end").about("Ends the current task"))
        .subcommand(Command::new("list").about("Lists the previous taks"))
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => cmd_start(sub_matches.value_of("NAME").expect("required")),
        Some(("end", _)) => cmd_end(),
        Some(("list", _)) => cmd_list(),
        _ => unreachable!(),
    }
}

fn get_storage() -> SqlStorage {
    let proj_dirs = ProjectDirs::from("Zeitfresser", "Zeitfresser", "Zeitfresser").unwrap();
    let config_dir = proj_dirs.config_dir();
    let db_path = config_dir.join("database.sqlite3");
    std::fs::create_dir_all(&config_dir).unwrap();

    SqlStorage::file(db_path)
}

fn cmd_start(name: &str) {
    let storage = get_storage();
    let e = data::Entry::new(name.to_string());
    storage.add_entry(&e);
}

fn cmd_end() {
    let storage = get_storage();

    let entries = match storage.current_entries() {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut last = match entries.last() {
        None => return,
        Some(e) => e.clone(),
    };

    last.end = Some(Utc::now());
    _ = storage.update_entry(&last);
}

fn cmd_list() {
    let storage = get_storage();
    for entry in storage.current_entries().unwrap() {
        println!("{:?}", entry);
    }
}
