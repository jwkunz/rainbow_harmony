use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig, SupportedStreamConfig};

use crate::cli::{CliCommand, ProgressionMode, ProgressionPlaybackOptions};
use crate::error::RainbowError;
use crate::synth::{SynthOptions, render_waveform};

struct PlaybackState {
    samples: Vec<f32>,
    index: usize,
    finished: bool,
}

pub fn play_waveform_blocking(
    chord: &crate::chord::Chord,
    duration: Duration,
    synth_options: SynthOptions,
) -> Result<(), RainbowError> {
    let (device, supported_config, config, sample_rate) = prepare_output()?;
    let samples = render_waveform(chord, duration, sample_rate, synth_options);
    play_samples_blocking(&device, &supported_config, &config, samples)
}

pub fn play_progression_blocking(
    commands: &[CliCommand],
    options: ProgressionPlaybackOptions,
) -> Result<(), RainbowError> {
    match options.mode {
        ProgressionMode::Separate => play_progression_separate(commands, options.gap_ms),
        ProgressionMode::Stitched => {
            let (device, supported_config, config, sample_rate) = prepare_output()?;
            let samples = render_progression_waveform(commands, sample_rate, options);
            play_samples_blocking(&device, &supported_config, &config, samples)
        }
    }
}

fn play_progression_separate(commands: &[CliCommand], gap_ms: u32) -> Result<(), RainbowError> {
    for (index, command) in commands.iter().enumerate() {
        play_waveform_blocking(&command.chord, command.duration, command.synth_options)?;

        if gap_ms > 0 && index + 1 < commands.len() {
            std::thread::sleep(Duration::from_millis(u64::from(gap_ms)));
        }
    }

    Ok(())
}

fn render_progression_waveform(
    commands: &[CliCommand],
    sample_rate: u32,
    options: ProgressionPlaybackOptions,
) -> Vec<f32> {
    let mut combined = Vec::new();
    let gap_samples = millis_to_samples(options.gap_ms, sample_rate);
    let legato_samples = millis_to_samples(options.legato_ms, sample_rate);

    for command in commands {
        let next = render_waveform(
            &command.chord,
            command.duration,
            sample_rate,
            command.synth_options,
        );

        if combined.is_empty() {
            combined = next;
            continue;
        }

        let overlap_samples = legato_samples.saturating_sub(gap_samples);
        let silence_samples = gap_samples.saturating_sub(legato_samples);

        if overlap_samples > 0 {
            let overlap = overlap_samples.min(combined.len()).min(next.len());
            crossfade_append(&mut combined, &next, overlap);
        } else {
            combined.extend(std::iter::repeat_n(0.0_f32, silence_samples));
            combined.extend(next);
        }
    }

    combined
}

fn millis_to_samples(millis: u32, sample_rate: u32) -> usize {
    ((millis as u64 * sample_rate as u64) / 1000) as usize
}

fn crossfade_append(output: &mut Vec<f32>, next: &[f32], overlap: usize) {
    if overlap == 0 {
        output.extend_from_slice(next);
        return;
    }

    let start = output.len() - overlap;
    for index in 0..overlap {
        let fade_out = 1.0 - index as f32 / overlap as f32;
        let fade_in = index as f32 / overlap as f32;
        output[start + index] = output[start + index] * fade_out + next[index] * fade_in;
    }

    output.extend_from_slice(&next[overlap..]);
}

fn prepare_output() -> Result<(cpal::Device, SupportedStreamConfig, StreamConfig, u32), RainbowError> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| RainbowError::audio("no default output audio device is available"))?;

    let supported_config = device.default_output_config().map_err(|error| {
        RainbowError::audio(format!("failed to read default output config: {error}"))
    })?;

    let sample_rate = supported_config.sample_rate().0;
    let config: StreamConfig = supported_config.config();
    Ok((device, supported_config, config, sample_rate))
}

fn play_samples_blocking(
    device: &cpal::Device,
    supported_config: &SupportedStreamConfig,
    config: &StreamConfig,
    samples: Vec<f32>,
) -> Result<(), RainbowError> {
    let playback_state = Arc::new((
        Mutex::new(PlaybackState {
            samples,
            index: 0,
            finished: false,
        }),
        Condvar::new(),
    ));

    let error_state = Arc::new(Mutex::new(None::<String>));
    let stream = match supported_config.sample_format() {
        SampleFormat::F32 => build_stream::<f32>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::F64 => build_stream::<f64>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::I8 => build_stream::<i8>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::I16 => build_stream::<i16>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::I24 => build_stream::<cpal::I24>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::I32 => build_stream::<i32>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::I64 => build_stream::<i64>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::U8 => build_stream::<u8>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::U16 => build_stream::<u16>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::U32 => build_stream::<u32>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        SampleFormat::U64 => build_stream::<u64>(
            device,
            config,
            playback_state.clone(),
            error_state.clone(),
        )?,
        sample_format => {
            return Err(RainbowError::audio(format!(
                "unsupported sample format: {sample_format:?}"
            )));
        }
    };

    stream
        .play()
        .map_err(|error| RainbowError::audio(format!("failed to start audio stream: {error}")))?;

    let (mutex, cvar) = &*playback_state;
    let mut state = mutex.lock().expect("playback state mutex poisoned");

    while !state.finished {
        state = cvar
            .wait(state)
            .expect("playback state mutex poisoned while waiting");
    }

    drop(state);
    drop(stream);

    if let Some(message) = error_state.lock().expect("error mutex poisoned").take() {
        return Err(RainbowError::audio(message));
    }

    Ok(())
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    playback_state: Arc<(Mutex<PlaybackState>, Condvar)>,
    error_state: Arc<Mutex<Option<String>>>,
) -> Result<cpal::Stream, RainbowError>
where
    T: Sample + SizedSample + FromSample<f32>,
{
    let channels = usize::from(config.channels);
    let playback_state_for_data = playback_state.clone();
    let playback_state_for_error = playback_state.clone();
    device
        .build_output_stream(
            config,
            move |data: &mut [T], _| write_data(data, channels, &playback_state_for_data),
            move |error| {
                if let Ok(mut slot) = error_state.lock() {
                    *slot = Some(format!("audio stream error: {error}"));
                }

                let (mutex, cvar) = &*playback_state_for_error;
                if let Ok(mut state) = mutex.lock() {
                    state.finished = true;
                    cvar.notify_all();
                }
            },
            None,
        )
        .map_err(|error| RainbowError::audio(format!("failed to build output stream: {error}")))
}

fn write_data<T>(
    output: &mut [T],
    channels: usize,
    playback_state: &Arc<(Mutex<PlaybackState>, Condvar)>,
) where
    T: Sample + FromSample<f32>,
{
    let (mutex, cvar) = &**playback_state;
    let mut state = mutex.lock().expect("playback state mutex poisoned");

    for frame in output.chunks_mut(channels) {
        let value = if state.index < state.samples.len() {
            let sample = state.samples[state.index];
            state.index += 1;

            if state.index >= state.samples.len() {
                state.finished = true;
                cvar.notify_all();
            }

            sample
        } else {
            state.finished = true;
            0.0
        };

        let converted = T::from_sample(value);
        for sample in frame {
            *sample = converted;
        }
    }
}
