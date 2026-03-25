use std::time::Duration;

use crate::chord::Chord;
use crate::error::RainbowError;

pub const DEFAULT_BPM: u32 = 120;

#[derive(Debug, Clone, PartialEq)]
pub struct CliCommand {
    pub chord: Chord,
    pub duration: Duration,
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
    let mut notes = None::<String>;
    let mut beats = None::<u32>;
    let mut bpm = DEFAULT_BPM;
    let mut seconds = None::<f64>;

    if remaining.is_empty() {
        return Err(RainbowError::usage(
            "missing arguments; expected flag-based usage",
        ));
    }

    let mut index = 0;
    while index < remaining.len() {
        match remaining[index].as_str() {
            "--help" | "-h" => {
                if remaining.len() == 1 {
                    return Ok(ParseOutcome::Help);
                }

                return Err(RainbowError::usage(
                    "--help must be used by itself",
                ));
            }
            "--notes" => {
                let value = flag_value(&remaining, &mut index, "--notes")?;
                notes = Some(value.to_string());
            }
            "--beats" => {
                let value = flag_value(&remaining, &mut index, "--beats")?;
                beats = Some(parse_positive_integer(value, "beats")?);
            }
            "--bpm" => {
                let value = flag_value(&remaining, &mut index, "--bpm")?;
                bpm = parse_positive_integer(value, "bpm")?;
            }
            "--seconds" => {
                let value = flag_value(&remaining, &mut index, "--seconds")?;
                seconds = Some(parse_positive_seconds(value)?);
            }
            flag if flag.starts_with("--") => {
                return Err(RainbowError::usage(format!("unknown flag `{flag}`")));
            }
            value => {
                return Err(RainbowError::usage(format!(
                    "unexpected positional argument `{value}`; this CLI now requires flags"
                )));
            }
        }

        index += 1;
    }

    let notes = notes.ok_or_else(|| RainbowError::usage("missing required flag --notes"))?;
    let chord = Chord::parse(&notes)?;

    let duration = match (beats, seconds) {
        (Some(beats), None) => Duration::from_secs_f64(beats as f64 * 60.0 / bpm as f64),
        (None, Some(seconds)) => {
            if bpm != DEFAULT_BPM {
                return Err(RainbowError::usage(
                    "--bpm may only be used together with --beats",
                ));
            }

            Duration::from_secs_f64(seconds)
        }
        (Some(_), Some(_)) => {
            return Err(RainbowError::usage(
                "use exactly one of --beats or --seconds",
            ));
        }
        (None, None) => {
            return Err(RainbowError::usage(
                "missing duration; use exactly one of --beats or --seconds",
            ));
        }
    };

    Ok(ParseOutcome::Run(CliCommand { chord, duration }))
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

fn parse_positive_seconds(input: &str) -> Result<f64, RainbowError> {
    let value: f64 = input.parse().map_err(|_| {
        RainbowError::usage(format!(
            "invalid seconds `{input}`; expected a positive number"
        ))
    })?;

    if !value.is_finite() || value <= 0.0 {
        return Err(RainbowError::usage(format!(
            "invalid seconds `{input}`; expected a positive number"
        )));
    }

    Ok(value)
}

fn flag_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag_name: &str,
) -> Result<&'a str, RainbowError> {
    let value_index = *index + 1;
    let value = args.get(value_index).ok_or_else(|| {
        RainbowError::usage(format!("missing value for {flag_name}"))
    })?;

    if value.starts_with("--") {
        return Err(RainbowError::usage(format!(
            "missing value for {flag_name}"
        )));
    }

    *index = value_index;
    Ok(value)
}

pub fn usage(program_name: &str) -> String {
    format!(
        "\
rainbow_harmony plays a note or chord using a flag-based CLI.

Usage:
  {program_name} --notes <notes> --beats <beats> [--bpm <bpm>]
  {program_name} --notes <notes> --seconds <seconds>
  {program_name} --help

Required:
  --notes <notes>      Note or chord expression such as A4, Bb3, or C4+E4+G4

Duration:
  --beats <beats>      Positive integer beat count
  --seconds <seconds>  Positive duration in seconds
  --bpm <bpm>          Optional positive integer tempo, defaults to {DEFAULT_BPM}

Notes:
  - This CLI no longer accepts positional arguments
  - Note letters A through G are accepted in either case
  - Sharps use # and flats use b, for example C#4 or Bb3
  - Octaves must be between 0 and 8
  - Chord notes are joined with +
"
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ParseOutcome, parse_args};

    #[test]
    fn parses_valid_beat_arguments() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "C4+E4".to_string(),
            "--beats".to_string(),
            "2".to_string(),
            "--bpm".to_string(),
            "90".to_string(),
        ])
        .unwrap();

        let ParseOutcome::Run(command) = outcome else {
            panic!("expected run outcome");
        };

        assert_eq!(command.chord.notes().len(), 2);
        assert_eq!(command.duration, Duration::from_secs_f64(2.0 * 60.0 / 90.0));
    }

    #[test]
    fn parses_seconds_arguments() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--seconds".to_string(),
            "1.5".to_string(),
        ])
        .unwrap();

        let ParseOutcome::Run(command) = outcome else {
            panic!("expected run outcome");
        };

        assert_eq!(command.duration, Duration::from_secs_f64(1.5));
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
            "--notes".to_string(),
            "A4".to_string(),
            "--beats".to_string(),
            "0".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("invalid beats"));
    }

    #[test]
    fn rejects_positional_arguments() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "A4".to_string(),
            "2".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("requires flags"));
    }

    #[test]
    fn rejects_missing_duration_flag() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("missing duration"));
    }

    #[test]
    fn rejects_using_beats_and_seconds_together() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--beats".to_string(),
            "2".to_string(),
            "--seconds".to_string(),
            "1.0".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("exactly one of --beats or --seconds"));
    }

    #[test]
    fn rejects_bpm_with_seconds() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--seconds".to_string(),
            "1.0".to_string(),
            "--bpm".to_string(),
            "90".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--bpm may only be used together with --beats"));
    }
}
