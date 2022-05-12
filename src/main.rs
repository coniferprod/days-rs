use std::env;
use std::io::Error;

fn run(_args: &[String]) -> Result<(), &'static str> {

    Ok(())
}

fn main() -> Result<(), Error> {
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
