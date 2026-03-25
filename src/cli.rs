use std::time::Duration;

use crate::chord::Chord;
use crate::error::RainbowError;

pub const DEFAULT_BPM: u32 = 120;

#[derive(Debug, Clone, PartialEq)]
pub struct CliCommand {
    pub chord: Chord,
    pub beats: u32,
    pub bpm: u32,
}

impl CliCommand {
    pub fn duration(&self) -> Duration {
        let seconds = self.beats as f64 * 60.0 / self.bpm as f64;
        Duration::from_secs_f64(seconds)
    }
}

#[derive(Debug)]
pub enum ParseOutcome {
    Run(CliCommand),
    Help,
}

pub fn parse_args<I>(args: I) -> Result<ParseOutcome, RainbowError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _program = args.next().unwrap_or_else(|| "rainbow_harmony".to_string());
    let remaining: Vec<String> = args.collect();

    if remaining.len() == 1 && matches!(remaining[0].as_str(), "--help" | "-h") {
        return Ok(ParseOutcome::Help);
    }

    if remaining.is_empty() {
        return Err(RainbowError::usage(
            "missing arguments; expected <notes> <beats> [bpm]",
        ));
    }

    if !(2..=3).contains(&remaining.len()) {
        return Err(RainbowError::usage(format!(
            "expected 2 or 3 arguments, got {}; expected <notes> <beats> [bpm]",
            remaining.len()
        )));
    }

    let chord = Chord::parse(&remaining[0])?;
    let beats = parse_positive_integer(&remaining[1], "beats")?;
    let bpm = match remaining.get(2) {
        Some(value) => parse_positive_integer(value, "bpm")?,
        None => DEFAULT_BPM,
    };

    Ok(ParseOutcome::Run(CliCommand { chord, beats, bpm }))
}

fn parse_positive_integer(input: &str, field_name: &str) -> Result<u32, RainbowError> {
    let value: u32 = input.parse().map_err(|_| {
        RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a positive integer"
        ))
    })?;

    if value == 0 {
        return Err(RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a positive integer"
        )));
    }

    Ok(value)
}

pub fn usage(program_name: &str) -> String {
    format!(
        "\
rainbow_harmony plays a note or chord for a specified number of beats.

Usage:
  {program_name} <notes> <beats> [bpm]
  {program_name} --help

Arguments:
  <notes>  Note or chord expression such as A4, Bb3, or C4+E4+G4
  <beats>  Positive integer beat count
  [bpm]    Optional positive integer tempo, defaults to {DEFAULT_BPM}

Notes:
  - Note letters A through G are accepted in either case
  - Sharps use # and flats use b, for example C#4 or Bb3
  - Octaves must be between 0 and 8
  - Chord notes are joined with +
"
    )
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_BPM, ParseOutcome, parse_args};

    #[test]
    fn parses_valid_arguments() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "C4+E4".to_string(),
            "2".to_string(),
            "90".to_string(),
        ])
        .unwrap();

        let ParseOutcome::Run(command) = outcome else {
            panic!("expected run outcome");
        };

        assert_eq!(command.chord.notes().len(), 2);
        assert_eq!(command.beats, 2);
        assert_eq!(command.bpm, 90);
    }

    #[test]
    fn supplies_default_bpm() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "A4".to_string(),
            "3".to_string(),
        ])
        .unwrap();

        let ParseOutcome::Run(command) = outcome else {
            panic!("expected run outcome");
        };

        assert_eq!(command.bpm, DEFAULT_BPM);
    }

    #[test]
    fn recognizes_help_flag() {
        let outcome =
            parse_args(vec!["rainbow_harmony".to_string(), "--help".to_string()]).unwrap();
        assert!(matches!(outcome, ParseOutcome::Help));
    }

    #[test]
    fn rejects_zero_beats() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "A4".to_string(),
            "0".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("invalid beats"));
    }
}
