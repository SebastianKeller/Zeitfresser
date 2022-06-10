#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod db;

use chrono::Timelike;
use clap::{arg, Command};
use db::models::{Filter, Task};

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
        .subcommand(
            Command::new("stop")
                .about("Ends the current task")
                .arg(arg!([TIME] "When the task has been finished.")),
        )
        .subcommand(Command::new("show").about("Presents an overview of previous tasks"))
        .subcommand(Command::new("clear").about("Removes all taks"))
}

fn main() {
    let matches = cli().get_matches();
    match matches.subcommand() {
        Some(("start", sub_matches)) => cmd_start(sub_matches.value_of("NAME").expect("")),
        Some(("stop", sub_matches)) => cmd_stop(sub_matches.value_of("TIME")),
        Some(("list", _)) => cmd_list(),
        Some(("clear", _)) => cmd_clear(),
        Some(("show", _)) => cmd_summary(),
        Some(r) => println!("Unknown command {}!", r.0),
        None => unreachable!(),
    }
}

fn db() -> db::DB {
    db::DB::new_xdg()
}

fn cmd_start(name: &str) {
    let db = db();
    db.finish_all();
    db.add_task(name);
}

fn cmd_stop(time: Option<&str>) {
    let db = db();
    let tasks = db.get_tasks(Filter::Last);
    let last_task = tasks.first();
    if last_task.is_none() {
        return;
    }
    let mut last_task = last_task.unwrap().clone();

    let date_time = match time {
        Some(s) => {
            let result = chrono::NaiveTime::parse_from_str(s, "%H:%M");
            if result.is_err() {
                println!("Could not parse {} to local time", s);
                return;
            }
            let result = result.unwrap();
            let mut date = chrono::Utc::now().naive_local();
            date = date.with_hour(result.hour()).unwrap();
            date = date.with_minute(result.minute()).unwrap();
            date = date.with_second(0).unwrap();
            date = date.with_nanosecond(0).unwrap();
            date
        }
        None => chrono::Utc::now().naive_local(),
    };

    last_task.finished_at = Some(date_time);
    db.update_task(last_task)
}

fn cmd_list() {
    let tasks = db().get_tasks(Filter::Day(chrono::Local::now().date()));
    print_tasks(tasks);
}

fn cmd_clear() {
    db().clear_tasks()
}

fn cmd_summary() {
    let tasks = db().get_tasks(Filter::Week);
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

        // Early return if we are about to print the same date
        if let Some(prev) = prev_date {
            if prev == date {
                return;
            }
        }

        let date_str = format!("{}:", date);
        let underscore: String = date_str.chars().map(|_| '-').collect();
        let new_line = match prev_date {
            Some(d) if d != date => "\n",
            Some(_) => "",
            None => "",
        };

        println!("{}{}\n{}", new_line, date_str, underscore);
        prev_date = Some(date);
    };

    for task in tasks {
        print_date_maybe(&task);

        let start = task.started_at.format("%H:%M:%S");
        let end = match task.finished_at {
            Some(value) => format!("{}", value.format("%H:%M:%S")).to_string(),
            None => "--:--:--".to_string(),
        };

        let num_seconds = task.duration().num_seconds();
        let seconds = num_seconds % 60;
        let minutes = (num_seconds / 60) % 60;
        let hours = (num_seconds / (60 * 60)) % 60;

        let duration = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);
        println!("{} - {} ({}) | {}", start, end, duration, task.title)
    }
}
