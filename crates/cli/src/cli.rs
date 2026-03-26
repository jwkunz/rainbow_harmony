use rainbow_harmony_core::error::RainbowError;
pub use rainbow_harmony_core::progression::{
    DEFAULT_BPM, DEFAULT_SECONDS, PlaybackEvent as CliCommand, ProgressionMode,
    ProgressionPlaybackOptions, parse_progression_text, parse_single_event_args,
};

#[derive(Debug)]
pub enum ParseOutcome {
    Run(CliCommand),
    RunProgression(Vec<CliCommand>, ProgressionPlaybackOptions),
    RunProgressionFile(String, ProgressionPlaybackOptions),
    Help,
}

pub fn parse_args<I>(args: I) -> Result<ParseOutcome, RainbowError>
where
    I: IntoIterator<Item = String>,
{
    let mut args = args.into_iter();
    let _program = args.next().unwrap_or_else(|| "rainbow_harmony".to_string());
    let remaining: Vec<String> = args.collect();

    if remaining.is_empty() {
        return Err(RainbowError::usage(
            "missing arguments; expected flag-based usage",
        ));
    }

    if remaining.len() == 1 && matches!(remaining[0].as_str(), "--help" | "-h") {
        return Ok(ParseOutcome::Help);
    }

    let mut progression = None::<String>;
    let mut progression_file = None::<String>;
    let mut progression_options = ProgressionPlaybackOptions::default();
    let mut saw_non_progression_flag = false;
    let mut saw_progression_option = false;
    let mut index = 0;
    while index < remaining.len() {
        match remaining[index].as_str() {
            "--progression" => {
                let value = raw_flag_value(&remaining, &mut index, "--progression")?;
                progression = Some(value.to_string());
            }
            "--progression-file" => {
                let value = raw_flag_value(&remaining, &mut index, "--progression-file")?;
                progression_file = Some(value.to_string());
            }
            "--progression-mode" => {
                let value = flag_value(&remaining, &mut index, "--progression-mode")?;
                progression_options.mode = parse_progression_mode(value)?;
                saw_progression_option = true;
            }
            "--gap-ms" => {
                let value = flag_value(&remaining, &mut index, "--gap-ms")?;
                progression_options.gap_ms = parse_non_negative_u32(value, "gap")?;
                saw_progression_option = true;
            }
            "--legato-ms" => {
                let value = flag_value(&remaining, &mut index, "--legato-ms")?;
                progression_options.legato_ms = parse_non_negative_u32(value, "legato")?;
                saw_progression_option = true;
            }
            "--help" | "-h" => {}
            flag if flag.starts_with("--") => {
                saw_non_progression_flag = true;
            }
            _ => {}
        }

        index += 1;
    }

    match (progression, progression_file) {
        (Some(_), Some(_)) => {
            return Err(RainbowError::usage(
                "use at most one of --progression or --progression-file",
            ));
        }
        (Some(text), None) => {
            if saw_non_progression_flag {
                return Err(RainbowError::usage(
                    "--progression cannot be combined with other event flags",
                ));
            }
            return Ok(ParseOutcome::RunProgression(
                parse_progression_text(&text)?,
                progression_options,
            ));
        }
        (None, Some(path)) => {
            if saw_non_progression_flag {
                return Err(RainbowError::usage(
                    "--progression-file cannot be combined with other event flags",
                ));
            }
            return Ok(ParseOutcome::RunProgressionFile(path, progression_options));
        }
        (None, None) => {
            if saw_progression_option {
                return Err(RainbowError::usage(
                    "--progression-mode, --gap-ms, and --legato-ms require --progression or --progression-file",
                ));
            }
        }
    }

    Ok(ParseOutcome::Run(parse_single_event_args(&remaining)?))
}

fn parse_non_negative_u32(input: &str, field_name: &str) -> Result<u32, RainbowError> {
    input.parse().map_err(|_| {
        RainbowError::usage(format!(
            "invalid {field_name} `{input}`; expected a non-negative integer"
        ))
    })
}

fn parse_progression_mode(input: &str) -> Result<ProgressionMode, RainbowError> {
    match input {
        "stitched" => Ok(ProgressionMode::Stitched),
        "separate" => Ok(ProgressionMode::Separate),
        _ => Err(RainbowError::usage(format!(
            "invalid progression mode `{input}`; expected stitched or separate"
        ))),
    }
}

fn flag_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag_name: &str,
) -> Result<&'a str, RainbowError> {
    let value_index = *index + 1;
    let value = args
        .get(value_index)
        .ok_or_else(|| RainbowError::usage(format!("missing value for {flag_name}")))?;

    if value.starts_with("--") {
        return Err(RainbowError::usage(format!(
            "missing value for {flag_name}"
        )));
    }

    *index = value_index;
    Ok(value)
}

fn raw_flag_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag_name: &str,
) -> Result<&'a str, RainbowError> {
    let value_index = *index + 1;
    let value = args
        .get(value_index)
        .ok_or_else(|| RainbowError::usage(format!("missing value for {flag_name}")))?;

    *index = value_index;
    Ok(value)
}

pub fn usage(program_name: &str) -> String {
    format!(
        "\
rainbow_harmony plays a note or chord using a flag-based CLI.

Usage:
  {program_name} (--notes <notes> | --chord <symbol>) [--beats <beats> [--bpm <bpm>] | --seconds <seconds>] [options]
  {program_name} --progression '<event => event => event>' [--progression-mode <mode>] [--gap-ms <ms>] [--legato-ms <ms>]
  {program_name} --progression-file <path> [--progression-mode <mode>] [--gap-ms <ms>] [--legato-ms <ms>]
  {program_name} --help

Required:
  --notes <notes>      Explicit note list such as A4 or C4+E4+G4
  --chord <symbol>     Chord symbol such as Cmaj7, Cm7b5, or C/E
  --progression <text> Inline progression with => separators and sticky settings
  --progression-file <path>
                       File containing a progression with => separators and sticky settings
  --progression-mode <mode>
                       Progression playback mode: stitched or separate, default stitched
  --gap-ms <ms>        Silence inserted between progression events, default 0
  --legato-ms <ms>     Boundary overlap between progression events, default 0

Chord Symbols:
  Triads:              C or Cmaj, Cm or Cmin, Caug or C+, Cdim or Co
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
  --harmonic-decay <rate>              Exponential harmonic falloff rate, default 0.8
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
  - Progressions cannot be combined with one-event flags at the top level
  - --bpm may only be used with --beats
  - Note letters A through G are accepted in either case
  - Sharps use # and flats use b, for example C#4 or Bb3
  - --notes requires octaves on every note segment
  - --chord accepts chord symbols such as Cm7, Cadd9, C7#5, or C/E
  - In progressions, => separates events and omitted settings stay sticky
  - stitched pre-renders the full progression into one buffer for smoother playback
  - separate plays each event individually and preserves audible boundaries
  - Octaves must be between 0 and 8
"
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rainbow_harmony_core::synth::SynthOptions;

    use super::{
        DEFAULT_SECONDS, ParseOutcome, ProgressionMode, ProgressionPlaybackOptions, parse_args,
    };

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
    fn parses_progression_with_sticky_settings() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--progression".to_string(),
            "--chord Cmaj7 --beats 2 --bpm 120 => --chord Cmin --beats 3".to_string(),
        ])
        .unwrap();

        let ParseOutcome::RunProgression(commands, options) = outcome else {
            panic!("expected progression outcome");
        };

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].duration, Duration::from_secs_f64(1.0));
        assert_eq!(commands[1].duration, Duration::from_secs_f64(1.5));
        assert_eq!(commands[1].synth_options, SynthOptions::default());
        assert_eq!(options, ProgressionPlaybackOptions::default());
    }

    #[test]
    fn progression_defaults_to_two_seconds_when_no_duration_is_set() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--progression".to_string(),
            "--notes C4+E4+G4 => --notes D4+F4+A4".to_string(),
        ])
        .unwrap();

        let ParseOutcome::RunProgression(commands, _) = outcome else {
            panic!("expected progression outcome");
        };

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].duration, Duration::from_secs_f64(DEFAULT_SECONDS));
        assert_eq!(commands[1].duration, Duration::from_secs_f64(DEFAULT_SECONDS));
    }

    #[test]
    fn parses_progression_playback_options() {
        let outcome = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--progression".to_string(),
            "--chord Cmaj7 => --chord F7".to_string(),
            "--progression-mode".to_string(),
            "separate".to_string(),
            "--gap-ms".to_string(),
            "25".to_string(),
            "--legato-ms".to_string(),
            "5".to_string(),
        ])
        .unwrap();

        let ParseOutcome::RunProgression(_, options) = outcome else {
            panic!("expected progression outcome");
        };

        assert_eq!(
            options,
            ProgressionPlaybackOptions {
                mode: ProgressionMode::Separate,
                gap_ms: 25,
                legato_ms: 5,
            }
        );
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

    #[test]
    fn rejects_progression_with_other_event_flags() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--progression".to_string(),
            "--chord Cmaj7".to_string(),
            "--beats".to_string(),
            "2".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("--progression cannot be combined"));
    }

    #[test]
    fn rejects_progression_options_without_progression() {
        let error = parse_args(vec![
            "rainbow_harmony".to_string(),
            "--notes".to_string(),
            "A4".to_string(),
            "--gap-ms".to_string(),
            "10".to_string(),
        ])
        .unwrap_err()
        .to_string();

        assert!(error.contains("require --progression or --progression-file"));
    }
}
