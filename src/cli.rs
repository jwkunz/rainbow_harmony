use std::time::Duration;

use crate::chord::Chord;
use crate::error::RainbowError;
use crate::synth::SynthOptions;

pub const DEFAULT_BPM: u32 = 120;
pub const DEFAULT_SECONDS: f64 = 2.0;

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
    let mut chord_name = None::<String>;
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
            "--chord" => {
                let value = flag_value(&remaining, &mut index, "--chord")?;
                chord_name = Some(value.to_string());
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
            "--harmonic-decay" => {
                let value = flag_value(&remaining, &mut index, "--harmonic-decay")?;
                synth_options.harmonic_decay =
                    parse_non_negative_f32(value, "harmonic decay")?;
            }
            "--phase-mod-rate" => {
                let value = flag_value(&remaining, &mut index, "--phase-mod-rate")?;
                synth_options.phase_mod_rate_hz =
                    parse_non_negative_f32(value, "phase modulation rate")?;
            }
            "--phase-mod-depth" => {
                let value = flag_value(&remaining, &mut index, "--phase-mod-depth")?;
                synth_options.phase_mod_depth_radians =
                    parse_non_negative_f32(value, "phase modulation depth")?;
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
            "--vibrato-rate" => {
                let value = flag_value(&remaining, &mut index, "--vibrato-rate")?;
                synth_options.vibrato_rate_hz = parse_non_negative_f32(value, "vibrato rate")?;
            }
            "--vibrato-depth-cents" => {
                let value = flag_value(&remaining, &mut index, "--vibrato-depth-cents")?;
                synth_options.vibrato_depth_cents =
                    parse_non_negative_f32(value, "vibrato depth")?;
            }
            "--noise-depth" => {
                let value = flag_value(&remaining, &mut index, "--noise-depth")?;
                synth_options.noise_depth =
                    parse_inclusive_f32_range(value, "noise depth", 0.0, 1.0)?;
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

    let chord = match (notes, chord_name) {
        (Some(notes), None) => Chord::parse_notes(&notes)?,
        (None, Some(chord_name)) => Chord::parse_symbol_name(&chord_name)?,
        (Some(_), Some(_)) => {
            return Err(RainbowError::usage(
                "use exactly one of --notes or --chord",
            ));
        }
        (None, None) => {
            return Err(RainbowError::usage(
                "missing required input; use exactly one of --notes or --chord",
            ));
        }
    };

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
            if bpm != DEFAULT_BPM {
                return Err(RainbowError::usage(
                    "--bpm may only be used together with --beats",
                ));
            }

            Duration::from_secs_f64(DEFAULT_SECONDS)
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
  {program_name} (--notes <notes> | --chord <symbol>) [--beats <beats> [--bpm <bpm>] | --seconds <seconds>] [options]
  {program_name} --help

Required:
  --notes <notes>      Explicit note list such as A4 or C4+E4+G4
  --chord <symbol>     Chord symbol such as Cmaj7, Cm7b5, or C/E

Chord Symbols:
  Triads:              major, minor, augmented, diminished
  Extensions:          6, 7, maj7, 9, 11, 13
  Added tones:         add2, add4, add9
  Suspensions:         sus2, sus4, sus
  Alterations:         b5, #5, b9, #9, #11, b13
  Bass notation:       slash chords such as C/E

Duration:
  --beats <beats>      Positive integer beat count
  --seconds <seconds>  Positive duration in seconds, default {DEFAULT_SECONDS}
  --bpm <bpm>          Optional positive integer tempo, defaults to {DEFAULT_BPM}

Synthesis:
  --gain <gain>                        Output gain before normalization, default 1.0
  --harmonics <count>                  Harmonic count including the fundamental, default 33
  --harmonic-decay <rate>             Exponential harmonic falloff rate, default 0.8
  --phase-mod-rate <hz>                Phase modulation rate in Hz, default 1000.0
  --phase-mod-depth <radians>          Phase modulation depth in radians, default 0.05
  --amp-tremolo-rate <hz>              Amplitude tremolo rate in Hz, default 3.0
  --amp-tremolo-depth <depth>          Amplitude tremolo depth from 0.0 to 1.0, default 0.05
  --vibrato-rate <hz>                  Vibrato rate in Hz, default 5.0
  --vibrato-depth-cents <cents>        Vibrato depth in cents, default 0.25
  --noise-depth <depth>                White noise depth from 0.0 to 1.0, default 0.02
  --seed <integer>                     Seed for reproducible phase variation, default 0

Notes:
  - This CLI no longer accepts positional arguments
  - If neither --beats nor --seconds is provided, playback defaults to 2.0 seconds
  - Do not use --beats and --seconds together
  - Use exactly one of --notes or --chord
  - --bpm may only be used with --beats
  - Note letters A through G are accepted in either case
  - Sharps use # and flats use b, for example C#4 or Bb3
  - --notes requires octaves on every note segment
  - --chord accepts chord symbols such as Cm7, Cadd9, C7#5, or C/E
  - Octaves must be between 0 and 8
"
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{DEFAULT_SECONDS, ParseOutcome, SynthOptions, parse_args};

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
    fn defaults_to_two_seconds_when_duration_is_omitted() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
        ])
        .unwrap();

        let ParseOutcome::Run(command) = outcome else {
            panic!("expected run outcome");
        };

        assert_eq!(command.duration, Duration::from_secs_f64(DEFAULT_SECONDS));
    }

    #[test]
    fn parses_synthesis_flags() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--chord".to_string(),
            "A".to_string(),
            "--seconds".to_string(),
            "1.5".to_string(),
            "--gain".to_string(),
            "0.8".to_string(),
            "--harmonics".to_string(),
            "12".to_string(),
            "--harmonic-decay".to_string(),
            "0.2".to_string(),
            "--phase-mod-rate".to_string(),
            "0.5".to_string(),
            "--phase-mod-depth".to_string(),
            "1.57".to_string(),
            "--amp-tremolo-rate".to_string(),
            "2.0".to_string(),
            "--amp-tremolo-depth".to_string(),
            "0.25".to_string(),
            "--vibrato-rate".to_string(),
            "8.0".to_string(),
            "--vibrato-depth-cents".to_string(),
            "15.0".to_string(),
            "--noise-depth".to_string(),
            "0.05".to_string(),
            "--seed".to_string(),
            "42".to_string(),
        ])
        .unwrap();

        let ParseOutcome::Run(command) = outcome else {
            panic!("expected run outcome");
        };

        assert_eq!(command.synth_options.gain, 0.8);
        assert_eq!(command.synth_options.harmonics, 12);
        assert_eq!(command.synth_options.harmonic_decay, 0.2);
        assert_eq!(command.synth_options.phase_mod_rate_hz, 0.5);
        assert_eq!(command.synth_options.phase_mod_depth_radians, 1.57);
        assert_eq!(command.synth_options.amp_tremolo_rate_hz, 2.0);
        assert_eq!(command.synth_options.amp_tremolo_depth, 0.25);
        assert_eq!(command.synth_options.vibrato_rate_hz, 8.0);
        assert_eq!(command.synth_options.vibrato_depth_cents, 15.0);
        assert_eq!(command.synth_options.noise_depth, 0.05);
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
    fn rejects_bpm_without_beats() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--bpm".to_string(),
            "90".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--bpm may only be used together with --beats"));
    }

    #[test]
    fn rejects_using_notes_and_chord_together() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--chord".to_string(),
            "Cmaj7".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("exactly one of --notes or --chord"));
    }

    #[test]
    fn rejects_missing_notes_or_chord() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--seconds".to_string(),
            "1.0".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("exactly one of --notes or --chord"));
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

    #[test]
    fn rejects_out_of_range_noise_depth() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--seconds".to_string(),
            "1.0".to_string(),
            "--noise-depth".to_string(),
            "1.5".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("noise depth"));
    }

    #[test]
    fn rejects_negative_harmonic_decay() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--seconds".to_string(),
            "1.0".to_string(),
            "--harmonic-decay".to_string(),
            "-0.1".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("harmonic decay"));
    }
}
