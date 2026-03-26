use std::env;
use std::fs;
use std::process::ExitCode;

use rainbow_harmony::audio::{play_progression_blocking, play_waveform_blocking};
use rainbow_harmony::cli::{ParseOutcome, parse_args, parse_progression_text, usage};
use rainbow_harmony_core::describe::format_event_description;

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
        Ok(ParseOutcome::Run(command, execution)) => {
            if execution.describe {
                println!("{}", format_event_description(&command));
            }

            if execution.silent {
                return Ok(());
            }

            play_waveform_blocking(&command.chord, command.duration, command.synth_options)
                .map_err(|error| error.to_string())
        }
        Ok(ParseOutcome::RunProgression(commands, options, execution)) => {
            if execution.describe {
                print_progression_descriptions(&commands);
            }

            if execution.silent {
                return Ok(());
            }

            play_progression_blocking(&commands, options).map_err(|error| error.to_string())
        }
        Ok(ParseOutcome::RunProgressionFile(path, options, execution)) => {
            let text = fs::read_to_string(&path)
                .map_err(|error| format!("failed to read progression file `{path}`: {error}"))?;
            let commands = parse_progression_text(&text).map_err(|error| error.to_string())?;

            if execution.describe {
                print_progression_descriptions(&commands);
            }

            if execution.silent {
                return Ok(());
            }

            play_progression_blocking(&commands, options).map_err(|error| error.to_string())
        }
        Err(error) => Err(format!("{error}\n\n{}", usage(&program_name))),
    }
}

fn print_progression_descriptions(commands: &[rainbow_harmony_core::progression::PlaybackEvent]) {
    for (index, command) in commands.iter().enumerate() {
        println!("step {}:", index + 1);
        println!("{}", format_event_description(command));
    }
}
