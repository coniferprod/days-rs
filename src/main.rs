use std::env;
use std::io;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::fmt;
use chrono::{NaiveDate, Datelike, DateTime, Local};
use csv::{Writer, ReaderBuilder};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Event {
    date: NaiveDate,
    category: String,
    description: String,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}: {} ({})", self.date.to_string(), self.description, self.category)
    }
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
    InvalidDate,
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
            },
            DaysError::InvalidDate => {
                write!(f, "Invalid date")
            },
        }
    }
}

impl std::error::Error for DaysError { }

fn run(_args: &[String]) -> Result<(), DaysError> {
    print_birthday();

    let mut events: Vec<Event> = Vec::new();

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
                eprintln!("Error reading events");
                return Err(DaysError::ReadError);
            }

            for event in events.iter() {
                println!("{}", event);
            }
        }

        Ok(())
    }
    else {
        eprintln!(".days path not found!");
        return Err(DaysError::WorkingDirectoryNotFound)
    }
}

fn read_events(events: &mut Vec<Event>, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().has_headers(true).from_path(path)?;
    for result in reader.records() {
        let record = result?;
        let category = record[1].to_string();
        let description = record[2].to_string();
        match NaiveDate::parse_from_str(&record[0], "%Y-%m-%d") {
            Ok(date) => {
                events.push(Event { date, category, description });
            },
            Err(_) => {
                eprintln!("Invalid timestamp '{}'", record[0].to_string());
            }
        }
    }
    Ok(())
}

fn write_events(events: Vec<Event>, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut writer = Writer::from_path(path)?;
    writer.write_record(&["date", "category", "description"])?;
    for event in events.iter() {
        writer.write_record(&[event.date.to_string(), event.category.clone(), event.description.clone()])?;
    }
    writer.flush()?;
    Ok(())
}

// See https://blog.liw.fi/posts/2021/10/12/tilde-expansion-crates/ for notes.

fn get_days_path() -> Option<PathBuf> {
    // NOTE: Don't use std::env::home_dir to get the home directory,
    // it doesn't work like it should! Use the `dirs` crate instead.
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
    if let Ok(value) = env::var("BIRTHDATE") {
        match NaiveDate::parse_from_str(&value, "%Y-%m-%d") {
            Ok(birthdate) => {
                let now: DateTime<Local> = Local::now();
                match NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()) {
                    Some(now_date) => {
                        if birthdate.month() == now_date.month() && birthdate.day() == now_date.day() {
                            print!("Happy birthday! ");
                        }
                        let diff = now_date.signed_duration_since(birthdate);
                        let day_count = diff.num_days();
                        print!("You are {} days old.", day_count);
                        if day_count % 1000 == 0 {
                            print!(" That's a nice round number!");
                        }
                        println!();
                    },
                    None => {
                        eprintln!("Unable to get today's date");
                    }
                }
            },
            Err(_) => {
                eprintln!("Error in the value of the BIRTHDATE environment variable: \
                    '{}' is not a valid date.", value);
            }
        };
    }
}

fn main() -> Result<(), DaysError> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    let result = run(&args[1..]);
    std::process::exit(match result {
        Ok(_) => exitcode::OK,
        Err(err) => {
            eprintln!("error: {:?}", err);
            exitcode::USAGE
        }
    });
}
