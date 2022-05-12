use std::env;
use std::io;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fmt;
use chrono::prelude::*;
use chrono::{NaiveDate, Datelike, DateTime, Local, Utc, TimeZone};
use csv::{Writer, ReaderBuilder};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Event {
    timestamp: u64,
    description: String,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct EventItem {
    days: i64,
    event: Event,
}

#[derive(Debug, Clone)]
enum DaysError {
    HomeDirectoryNotFound,
    WorkingDirectoryNotFound,
    CreateError,
    WriteError,
    ReadError,
}

impl fmt::Display for DaysError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            DaysError::HomeDirectoryNotFound => {
                write!(f, "Home directory not found")
            },
            DaysError::WorkingDirectoryNotFound => {
                write!(f, "Working directory not found")
            },
            DaysError::ReadError => {
                write!(f, "Error reading events")
            },
            DaysError::WriteError => {
                write!(f, "Error writing events")
            },
            DaysError::CreateError => {
                write!(f, "Unable to crate working directory")
            }
        }
    }
}

impl std::error::Error for DaysError { }

fn run(_args: &[String]) -> Result<(), DaysError> {
    print_birthday();

    let mut events: Vec<Event> = Vec::new();
    let mut past_items: Vec<EventItem> = Vec::new();
    let mut future_items: Vec<EventItem> = Vec::new();

    if let Some(path) = get_days_path() {
        // Create the working directory if it does not exist.
        if !Path::exists(path.as_path()) {
            match std::fs::create_dir(path.as_path()) {
                Ok(_) => {},
                Err(_) => {
                    return Err(DaysError::CreateError);
                }
            }
        }

        let mut events_path = path.clone();
        events_path.push("events.csv");

        if events_path.as_path().exists() {
            // Read in the events
            if let Err(_) = read_events(&mut events, events_path.as_path()) {
                return Err(DaysError::ReadError);
            }

            let today: DateTime<Local> = Local::now();

            for event in events {

                let event_dt = Utc.timestamp(event.timestamp as i64, 0);

                let diff = event_dt.signed_duration_since(today);
                let day_count = diff.num_days();
                if day_count <= 0 {
                    past_items.push(EventItem { days: day_count, event });
                }
                else {
                    future_items.push(EventItem { days: day_count, event });
                }
            }
        }
        else {
            // Create the file with one seed event:
            let now: DateTime<Utc> = Utc::now();

            events.push(Event { timestamp: now.timestamp() as u64, description: "Started to use the days program.".to_string()});

            if let Err(_) = write_events(events, events_path.as_path()) {
                return Err(DaysError::WriteError);
            }
        }

        past_items.sort_by(|a, b| b.days.cmp(&a.days));
        println!("Past events");
        println!("-----------");
        for item in past_items {
            println!("{} days ago\t{}", item.days.abs(), item.event.description);
        }

        future_items.sort_by(|a, b| a.days.cmp(&b.days));
        println!("\nUpcoming events");
        println!("---------------");
        for item in future_items {
            println!("In {} days\t{}", item.days.abs(), item.event.description);
        }
        println!();

        Ok(())
    }
    else {
        eprintln!(".days path not found!");
        return Err(DaysError::WorkingDirectoryNotFound)
    }
}

fn read_events(events: &mut Vec<Event>, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;
    events.clear();
    for result in reader.records() {
        let record = result?;
        let description = record[1].to_string();
        if let Some(timestamp) = record[0].parse().ok() {
            events.push(Event { timestamp, description });
        }
        else {
            eprintln!("Invalid timestamp '{}' in event '{}'", record[0].to_string(), description);
        }
    }
    Ok(())
}

fn write_events(events: Vec<Event>, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(&["timestamp", "description"])?;
    for event in events.iter() {
        writer.write_record(&[event.timestamp.to_string(), event.description.clone()])?;
    }
    writer.flush()?;
    Ok(())
}

// See https://blog.liw.fi/posts/2021/10/12/tilde-expansion-crates/ for notes.

fn get_days_path() -> Option<PathBuf> {
    // NOTE: Don't use std::env::home_dir to get the home directory,
    // it doesn't work like it should! Use the dirs crate instead.
    match dirs::home_dir() {
        Some(home_dir) => {
            let mut path = home_dir.clone();
            // Construct a path for the `~/.days` directory:
            path.push(".days");
            Some(path)
        },
        None => {
            eprintln!("No home directory for user");
            None
        }
    }
}

fn print_birthday() {
    if let Ok(value) = env::var("BIRTHDAY") {
        match NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
            Ok(birthday) => {
                let today: DateTime<Local> = Local::now();

                let birthday_dt = Local
                    .ymd(birthday.year(), birthday.month(), birthday.day())
                    .and_hms(today.time().hour(), today.time().minute(), today.time().second());

                let diff = today.signed_duration_since(birthday_dt);
                let day_count = diff.num_days();
                print!("You were born {} days ago.", day_count);

                if birthday.month() == today.month() && birthday.day() == today.day() {
                    print!(" Happy birthday to you!");
                }
                println!();
            },
            Err(_) => {
                eprintln!("Error in the value of the BIRTHDAY environment variable: \
                    '{}' is not a valid date.", value);
            }
        };
    }
}

fn main() -> Result<(), DaysError> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    std::process::exit(match run(&args[1..]) {
        Ok(_) => exitcode::OK,
        Err(err) => {
            eprintln!("error: {:?}", err);
            exitcode::USAGE
        }
    });
}
