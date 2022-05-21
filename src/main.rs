#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod db;
use clap::{arg, Command};

fn cli() -> Command<'static> {
    Command::new("zeitfresser")
        .about("A tool to track where your time is spent.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("start")
                .about("Tracks a new task, ending the previous one.")
                .arg(arg!(<NAME> "The name of the task"))
                .arg_required_else_help(true),
        )
        .subcommand(Command::new("end").about("Ends the current task"))
        .subcommand(Command::new("list").about("Lists the previous taks"))
        .subcommand(Command::new("summary"))
        .subcommand(Command::new("clear").about("Removes all taks"))
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => cmd_start(sub_matches.value_of("NAME").expect("")),
        Some(("end", _)) => cmd_end(),
        Some(("list", _)) => cmd_list(),
        Some(("clear", _)) => db::remove(),
        Some(("summary", _)) => cmd_summary(),
        _ => unreachable!(),
    }
}

fn cmd_start(name: &str) {
    db::end_all();
    db::add_task(name);
}

fn cmd_end() {
    db::end_all();
}

fn cmd_list() {
    let tasks = db::get_tasks();
    if tasks.is_empty() {
        println!("No tasks available.");
        return;
    }
    for task in tasks {
        println!("{:?}", task);
    }
}

fn cmd_summary() {
    let tasks = db::get_tasks();
    tasks.iter().for_each(|t| {
        let title = &t.title;
        let duration = t.finished_at.unwrap_or(chrono::Utc::now().naive_utc()) - t.started_at;
        println!(
            "{:02}:{:02}:{:02} - {}",
            duration.num_hours(),
            duration.num_minutes(),
            duration.num_seconds(),
            title
        )
    });
}
