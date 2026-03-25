use std::env;
use std::process::ExitCode;

use rainbow_harmony::audio::play_waveform_blocking;
use rainbow_harmony::cli::{ParseOutcome, parse_args, usage};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let program_name = env::args()
        .next()
        .unwrap_or_else(|| "rainbow_harmony".to_string());

    match parse_args(env::args()) {
        Ok(ParseOutcome::Help) => {
            println!("{}", usage(&program_name));
            Ok(())
        }
        Ok(ParseOutcome::Run(command)) => {
            play_waveform_blocking(&command.chord, command.duration)
                .map_err(|error| error.to_string())
        }
        Err(error) => Err(format!("{error}\n\n{}", usage(&program_name))),
    }
}
