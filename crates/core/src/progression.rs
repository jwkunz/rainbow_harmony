use std::time::Duration;

use crate::chord::Chord;
use crate::error::RainbowError;
use crate::synth::SynthOptions;

pub const DEFAULT_BPM: u32 = 120;
pub const DEFAULT_SECONDS: f64 = 2.0;

#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackEvent {
    pub chord: Chord,
    pub duration: Duration,
    pub synth_options: SynthOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressionMode {
    Stitched,
    Separate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressionPlaybackOptions {
    pub mode: ProgressionMode,
    pub gap_ms: u32,
    pub legato_ms: u32,
}

impl Default for ProgressionPlaybackOptions {
    fn default() -> Self {
        Self {
            mode: ProgressionMode::Stitched,
            gap_ms: 0,
            legato_ms: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum InputSpec {
    Notes(String),
    Chord(String),
}

#[derive(Debug, Clone, PartialEq)]
struct DurationState {
    beats: Option<u32>,
    seconds: Option<f64>,
    bpm: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct PartialCommand {
    input_spec: Option<InputSpec>,
    beats: Option<u32>,
    seconds: Option<f64>,
    bpm: Option<u32>,
    synth_options: SynthOptionsPatch,
}

#[derive(Debug, Clone, PartialEq, Default)]
struct SynthOptionsPatch {
    gain: Option<f32>,
    harmonics: Option<u32>,
    harmonic_decay: Option<f32>,
    phase_mod_rate_hz: Option<f32>,
    phase_mod_depth_radians: Option<f32>,
    amp_tremolo_rate_hz: Option<f32>,
    amp_tremolo_depth: Option<f32>,
    vibrato_rate_hz: Option<f32>,
    vibrato_depth_cents: Option<f32>,
    noise_depth: Option<f32>,
    seed: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
struct ProgressionState {
    input_spec: Option<InputSpec>,
    duration: DurationState,
    synth_options: SynthOptions,
}

impl Default for ProgressionState {
    fn default() -> Self {
        Self {
            input_spec: None,
            duration: DurationState {
                beats: None,
                seconds: Some(DEFAULT_SECONDS),
                bpm: DEFAULT_BPM,
            },
            synth_options: SynthOptions::default(),
        }
    }
}

impl ProgressionState {
    fn apply(&mut self, partial: PartialCommand) {
        if let Some(input_spec) = partial.input_spec {
            self.input_spec = Some(input_spec);
        }

        if let Some(beats) = partial.beats {
            self.duration.beats = Some(beats);
            self.duration.seconds = None;
        }

        if let Some(seconds) = partial.seconds {
            self.duration.seconds = Some(seconds);
            self.duration.beats = None;
        }

        if let Some(bpm) = partial.bpm {
            self.duration.bpm = bpm;
        }

        self.synth_options = merge_synth_options(self.synth_options, partial.synth_options);
    }

    fn to_event(&self) -> Result<PlaybackEvent, RainbowError> {
        let input_spec = self.input_spec.as_ref().ok_or_else(|| {
            RainbowError::usage("missing required input; use exactly one of --notes or --chord")
        })?;
        build_event_from_state(input_spec, &self.duration, self.synth_options)
    }
}

pub fn parse_single_event_args(args: &[String]) -> Result<PlaybackEvent, RainbowError> {
    let partial = parse_partial_command_args(args)?;
    build_single_event(partial)
}

pub fn parse_progression_text(text: &str) -> Result<Vec<PlaybackEvent>, RainbowError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(RainbowError::usage("progression cannot be empty"));
    }

    let mut state = ProgressionState::default();
    let mut events = Vec::new();

    for (index, segment) in trimmed.split("=>").enumerate() {
        let segment = segment.trim();
        if segment.is_empty() {
            return Err(RainbowError::usage(format!(
                "progression contains an empty segment at position {}",
                index + 1
            )));
        }

        let args = tokenize_progression_segment(segment);
        let partial = parse_partial_command_args(&args).map_err(|error| {
            RainbowError::usage(format!("progression segment {}: {error}", index + 1))
        })?;
        state.apply(partial);
        events.push(state.to_event().map_err(|error| {
            RainbowError::usage(format!("progression segment {}: {error}", index + 1))
        })?);
    }

    Ok(events)
}

fn build_single_event(partial: PartialCommand) -> Result<PlaybackEvent, RainbowError> {
    let input_spec = partial.input_spec.ok_or_else(|| {
        RainbowError::usage("missing required input; use exactly one of --notes or --chord")
    })?;
    if partial.bpm.is_some() && partial.beats.is_none() {
        return Err(RainbowError::usage(
            "--bpm may only be used together with --beats",
        ));
    }

    let duration = DurationState {
        beats: partial.beats,
        seconds: if partial.beats.is_some() {
            None
        } else {
            partial.seconds.or(Some(DEFAULT_SECONDS))
        },
        bpm: partial.bpm.unwrap_or(DEFAULT_BPM),
    };

    build_event_from_state(
        &input_spec,
        &duration,
        merge_synth_options(SynthOptions::default(), partial.synth_options),
    )
}

fn build_event_from_state(
    input_spec: &InputSpec,
    duration_state: &DurationState,
    synth_options: SynthOptions,
) -> Result<PlaybackEvent, RainbowError> {
    let chord = match input_spec {
        InputSpec::Notes(notes) => Chord::parse_notes(notes)?,
        InputSpec::Chord(chord_name) => Chord::parse_symbol_name(chord_name)?,
    };

    let duration = match (duration_state.beats, duration_state.seconds) {
        (Some(beats), None) => {
            Duration::from_secs_f64(beats as f64 * 60.0 / duration_state.bpm as f64)
        }
        (None, Some(seconds)) => Duration::from_secs_f64(seconds),
        (Some(_), Some(_)) => {
            return Err(RainbowError::usage(
                "use exactly one of --beats or --seconds",
            ));
        }
        (None, None) => Duration::from_secs_f64(DEFAULT_SECONDS),
    };

    Ok(PlaybackEvent {
        chord,
        duration,
        synth_options,
    })
}

fn merge_synth_options(base: SynthOptions, patch: SynthOptionsPatch) -> SynthOptions {
    SynthOptions {
        gain: patch.gain.unwrap_or(base.gain),
        harmonics: patch.harmonics.unwrap_or(base.harmonics),
        harmonic_decay: patch.harmonic_decay.unwrap_or(base.harmonic_decay),
        phase_mod_rate_hz: patch.phase_mod_rate_hz.unwrap_or(base.phase_mod_rate_hz),
        phase_mod_depth_radians: patch
            .phase_mod_depth_radians
            .unwrap_or(base.phase_mod_depth_radians),
        amp_tremolo_rate_hz: patch
            .amp_tremolo_rate_hz
            .unwrap_or(base.amp_tremolo_rate_hz),
        amp_tremolo_depth: patch.amp_tremolo_depth.unwrap_or(base.amp_tremolo_depth),
        vibrato_rate_hz: patch.vibrato_rate_hz.unwrap_or(base.vibrato_rate_hz),
        vibrato_depth_cents: patch
            .vibrato_depth_cents
            .unwrap_or(base.vibrato_depth_cents),
        noise_depth: patch.noise_depth.unwrap_or(base.noise_depth),
        seed: patch.seed.unwrap_or(base.seed),
    }
}

fn tokenize_progression_segment(segment: &str) -> Vec<String> {
    segment.split_whitespace().map(str::to_string).collect()
}

fn parse_partial_command_args(args: &[String]) -> Result<PartialCommand, RainbowError> {
    let mut notes = None::<String>;
    let mut chord_name = None::<String>;
    let mut beats = None::<u32>;
    let mut bpm = None::<u32>;
    let mut seconds = None::<f64>;
    let mut synth_options = SynthOptionsPatch::default();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--help" | "-h" => {
                return Err(RainbowError::usage("--help must be used by itself"));
            }
            "--notes" => {
                let value = flag_value(args, &mut index, "--notes")?;
                notes = Some(value.to_string());
            }
            "--chord" => {
                let value = flag_value(args, &mut index, "--chord")?;
                chord_name = Some(value.to_string());
            }
            "--beats" => {
                let value = flag_value(args, &mut index, "--beats")?;
                beats = Some(parse_positive_integer(value, "beats")?);
            }
            "--bpm" => {
                let value = flag_value(args, &mut index, "--bpm")?;
                bpm = Some(parse_positive_integer(value, "bpm")?);
            }
            "--seconds" => {
                let value = flag_value(args, &mut index, "--seconds")?;
                seconds = Some(parse_positive_seconds(value)?);
            }
            "--gain" => {
                let value = flag_value(args, &mut index, "--gain")?;
                synth_options.gain = Some(parse_positive_f32(value, "gain")?);
            }
            "--harmonics" => {
                let value = flag_value(args, &mut index, "--harmonics")?;
                synth_options.harmonics = Some(parse_positive_integer(value, "harmonics")?);
            }
            "--harmonic-decay" => {
                let value = flag_value(args, &mut index, "--harmonic-decay")?;
                synth_options.harmonic_decay = Some(parse_non_negative_f32(value, "harmonic decay")?);
            }
            "--phase-mod-rate" => {
                let value = flag_value(args, &mut index, "--phase-mod-rate")?;
                synth_options.phase_mod_rate_hz =
                    Some(parse_non_negative_f32(value, "phase modulation rate")?);
            }
            "--phase-mod-depth" => {
                let value = flag_value(args, &mut index, "--phase-mod-depth")?;
                synth_options.phase_mod_depth_radians =
                    Some(parse_non_negative_f32(value, "phase modulation depth")?);
            }
            "--amp-tremolo-rate" => {
                let value = flag_value(args, &mut index, "--amp-tremolo-rate")?;
                synth_options.amp_tremolo_rate_hz =
                    Some(parse_non_negative_f32(value, "amplitude tremolo rate")?);
            }
            "--amp-tremolo-depth" => {
                let value = flag_value(args, &mut index, "--amp-tremolo-depth")?;
                synth_options.amp_tremolo_depth = Some(parse_inclusive_f32_range(
                    value,
                    "amplitude tremolo depth",
                    0.0,
                    1.0,
                )?);
            }
            "--seed" => {
                let value = flag_value(args, &mut index, "--seed")?;
                synth_options.seed = Some(parse_u64(value, "seed")?);
            }
            "--vibrato-rate" => {
                let value = flag_value(args, &mut index, "--vibrato-rate")?;
                synth_options.vibrato_rate_hz =
                    Some(parse_non_negative_f32(value, "vibrato rate")?);
            }
            "--vibrato-depth-cents" => {
                let value = flag_value(args, &mut index, "--vibrato-depth-cents")?;
                synth_options.vibrato_depth_cents =
                    Some(parse_non_negative_f32(value, "vibrato depth")?);
            }
            "--noise-depth" => {
                let value = flag_value(args, &mut index, "--noise-depth")?;
                synth_options.noise_depth =
                    Some(parse_inclusive_f32_range(value, "noise depth", 0.0, 1.0)?);
            }
            "--progression"
            | "--progression-file"
            | "--progression-mode"
            | "--gap-ms"
            | "--legato-ms" => {
                return Err(RainbowError::usage(
                    "progression flags are not allowed inside event arguments",
                ));
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

    let input_spec = match (notes, chord_name) {
        (Some(notes), None) => Some(InputSpec::Notes(notes)),
        (None, Some(chord_name)) => Some(InputSpec::Chord(chord_name)),
        (Some(_), Some(_)) => {
            return Err(RainbowError::usage(
                "use exactly one of --notes or --chord",
            ));
        }
        (None, None) => None,
    };

    if beats.is_some() && seconds.is_some() {
        return Err(RainbowError::usage(
            "use exactly one of --beats or --seconds",
        ));
    }

    if bpm.is_some() && seconds.is_some() {
        return Err(RainbowError::usage(
            "--bpm may only be used together with --beats",
        ));
    }

    Ok(PartialCommand {
        input_spec,
        beats,
        seconds,
        bpm,
        synth_options,
    })
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
