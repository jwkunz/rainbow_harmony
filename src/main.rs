use std::env;
use std::fs;
use std::process::ExitCode;

use rainbow_harmony::audio::{play_progression_blocking, play_waveform_blocking};
use rainbow_harmony::cli::{ParseOutcome, parse_args, parse_progression_text, usage};

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
            play_waveform_blocking(&command.chord, command.duration, command.synth_options)
                .map_err(|error| error.to_string())
        }
        Ok(ParseOutcome::RunProgression(commands, options)) => {
            play_progression_blocking(&commands, options).map_err(|error| error.to_string())
        }
        Ok(ParseOutcome::RunProgressionFile(path, options)) => {
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read progression file `{path}`: {error}"))?;
            let commands = parse_progression_text(&text).map_err(|error| error.to_string())?;
            play_progression_blocking(&commands, options).map_err(|error| error.to_string())
        }
        Err(error) => Err(format!("{error}\n\n{}", usage(&program_name))),
    }
}
