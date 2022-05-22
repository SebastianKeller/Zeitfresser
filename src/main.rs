#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod db;
use clap::{arg, Command};
use db::models::Filter;
use db::models::Task;

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
        Some(("stop", _)) => cmd_stop(),
        Some(("list", _)) => cmd_list(),
        Some(("clear", _)) => db::remove(),
        Some(("summary", _)) => cmd_summary(),
        _ => unreachable!(),
    }
}

fn cmd_start(name: &str) {
    db::finish_all();
    db::add_task(name);
}

fn cmd_stop() {
    db::finish_all();
}

fn cmd_list() {
    let tasks = db::get_tasks(Filter::Day(chrono::Local::now().date()));
    print_tasks(tasks);
}

fn cmd_summary() {
    let tasks = db::get_tasks(Filter::Week);
    print_tasks(tasks);
}

fn print_tasks(tasks: Vec<Task>) {
    if tasks.is_empty() {
        println!("No tasks available.");
        return;
    }

    let mut prev_date: Option<chrono::NaiveDate> = None;
    let mut print_date_maybe = |task: &Task| {
        let date = chrono::DateTime::<chrono::Utc>::from_utc(task.started_at, chrono::Utc)
            .naive_local()
            .date();

        match prev_date {
            None => println!("{}:", date),
            Some(d) => {
                if d != date {
                    println!();
                    println!("{}:", date);
                }
            }
        }
        prev_date = Some(date);
    };

    for task in tasks {
        print_date_maybe(&task);

        let num_seconds = task.duration().num_seconds();
        let seconds = num_seconds % 60;
        let minutes = (num_seconds / 60) % 60;
        let hours = (num_seconds / (60 * 60)) % 60;

        let duration = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        println!("{} - {}", duration, task.title)
    }
}
