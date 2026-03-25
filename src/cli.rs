use std::time::Duration;

use crate::chord::Chord;
use crate::error::RainbowError;
use crate::synth::SynthOptions;

pub const DEFAULT_BPM: u32 = 120;

#[derive(Debug, Clone, PartialEq)]
pub struct CliCommand {
    pub chord: Chord,
    pub duration: Duration,
    pub synth_options: SynthOptions,
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
    let mut synth_options = SynthOptions::default();

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
            "--gain" => {
                let value = flag_value(&remaining, &mut index, "--gain")?;
                synth_options.gain = parse_positive_f32(value, "gain")?;
            }
            "--harmonics" => {
                let value = flag_value(&remaining, &mut index, "--harmonics")?;
                synth_options.harmonics = parse_positive_integer(value, "harmonics")?;
            }
            "--phase-tremolo-rate" => {
                let value = flag_value(&remaining, &mut index, "--phase-tremolo-rate")?;
                synth_options.phase_tremolo_rate_hz =
                    parse_non_negative_f32(value, "phase tremolo rate")?;
            }
            "--phase-tremolo-depth" => {
                let value = flag_value(&remaining, &mut index, "--phase-tremolo-depth")?;
                synth_options.phase_tremolo_depth_radians =
                    parse_non_negative_f32(value, "phase tremolo depth")?;
            }
            "--amp-tremolo-rate" => {
                let value = flag_value(&remaining, &mut index, "--amp-tremolo-rate")?;
                synth_options.amp_tremolo_rate_hz =
                    parse_non_negative_f32(value, "amplitude tremolo rate")?;
            }
            "--amp-tremolo-depth" => {
                let value = flag_value(&remaining, &mut index, "--amp-tremolo-depth")?;
                synth_options.amp_tremolo_depth =
                    parse_inclusive_f32_range(value, "amplitude tremolo depth", 0.0, 1.0)?;
            }
            "--seed" => {
                let value = flag_value(&remaining, &mut index, "--seed")?;
                synth_options.seed = parse_u64(value, "seed")?;
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

    Ok(ParseOutcome::Run(CliCommand {
        chord,
        duration,
        synth_options,
    }))
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

fn parse_positive_f32(input: &str, field_name: &str) -> Result<f32, RainbowError> {
    let value: f32 = input.parse().map_err(|_| {
        RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a positive number"
        ))
    })?;

    if !value.is_finite() || value <= 0.0 {
        return Err(RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a positive number"
        )));
    }

    Ok(value)
}

fn parse_non_negative_f32(input: &str, field_name: &str) -> Result<f32, RainbowError> {
    let value: f32 = input.parse().map_err(|_| {
        RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a non-negative number"
        ))
    })?;

    if !value.is_finite() || value < 0.0 {
        return Err(RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a non-negative number"
        )));
    }

    Ok(value)
}

fn parse_inclusive_f32_range(
    input: &str,
    field_name: &str,
    min: f32,
    max: f32,
) -> Result<f32, RainbowError> {
    let value: f32 = input.parse().map_err(|_| {
        RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a number from {min} to {max}"
        ))
    })?;

    if !value.is_finite() || value < min || value > max {
        return Err(RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a number from {min} to {max}"
        )));
    }

    Ok(value)
}

fn parse_u64(input: &str, field_name: &str) -> Result<u64, RainbowError> {
    input.parse().map_err(|_| {
        RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a non-negative integer"
        ))
    })
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
  {program_name} --notes <notes> (--beats <beats> [--bpm <bpm>] | --seconds <seconds>) [options]
  {program_name} --help

Required:
  --notes <notes>      Note or chord expression such as A4, Bb3, or C4+E4+G4

Duration:
  --beats <beats>      Positive integer beat count
  --seconds <seconds>  Positive duration in seconds
  --bpm <bpm>          Optional positive integer tempo, defaults to {DEFAULT_BPM}

Synthesis:
  --gain <gain>                        Output gain before normalization, default 1.0
  --harmonics <count>                  Harmonic count including the fundamental, default 33
  --phase-tremolo-rate <hz>            Phase modulation rate in Hz, default 1.0
  --phase-tremolo-depth <radians>      Phase modulation depth in radians, default 3.1415927
  --amp-tremolo-rate <hz>              Amplitude tremolo rate in Hz, default 1.0
  --amp-tremolo-depth <depth>          Amplitude tremolo depth from 0.0 to 1.0, default 0.1
  --seed <integer>                     Seed for reproducible phase variation, default 0

Notes:
  - This CLI no longer accepts positional arguments
  - Use exactly one of --beats or --seconds
  - --bpm may only be used with --beats
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

    use super::{ParseOutcome, SynthOptions, parse_args};

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
        assert_eq!(command.synth_options, SynthOptions::default());
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
    fn parses_synthesis_flags() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--seconds".to_string(),
            "1.5".to_string(),
            "--gain".to_string(),
            "0.8".to_string(),
            "--harmonics".to_string(),
            "12".to_string(),
            "--phase-tremolo-rate".to_string(),
            "0.5".to_string(),
            "--phase-tremolo-depth".to_string(),
            "1.57".to_string(),
            "--amp-tremolo-rate".to_string(),
            "2.0".to_string(),
            "--amp-tremolo-depth".to_string(),
            "0.25".to_string(),
            "--seed".to_string(),
            "42".to_string(),
        ])
        .unwrap();

        let ParseOutcome::Run(command) = outcome else {
            panic!("expected run outcome");
        };

        assert_eq!(command.synth_options.gain, 0.8);
        assert_eq!(command.synth_options.harmonics, 12);
        assert_eq!(command.synth_options.phase_tremolo_rate_hz, 0.5);
        assert_eq!(command.synth_options.phase_tremolo_depth_radians, 1.57);
        assert_eq!(command.synth_options.amp_tremolo_rate_hz, 2.0);
        assert_eq!(command.synth_options.amp_tremolo_depth, 0.25);
        assert_eq!(command.synth_options.seed, 42);
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

    #[test]
    fn rejects_out_of_range_amp_tremolo_depth() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--seconds".to_string(),
            "1.0".to_string(),
            "--amp-tremolo-depth".to_string(),
            "1.5".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("amplitude tremolo depth"));
    }
}
