use std::f32::consts::PI;
use std::time::Duration;

use crate::chord::Chord;

const FADE_TIME_SECONDS: f32 = 0.01;
const PHASE_GOLDEN_RATIO: f32 = 2.399_963_1;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SynthOptions {
    pub gain: f32,
    pub harmonics: u32,
    pub phase_mod_rate_hz: f32,
    pub phase_mod_depth_radians: f32,
    pub amp_tremolo_rate_hz: f32,
    pub amp_tremolo_depth: f32,
    pub vibrato_rate_hz: f32,
    pub vibrato_depth_cents: f32,
    pub seed: u64,
}

impl Default for SynthOptions {
    fn default() -> Self {
        Self {
            gain: 1.0,
            harmonics: 33,
            phase_mod_rate_hz: 1.0,
            phase_mod_depth_radians: PI,
            amp_tremolo_rate_hz: 1.0,
            amp_tremolo_depth: 0.1,
            vibrato_rate_hz: 10.0,
            vibrato_depth_cents: 3.0,
            seed: 0,
        }
    }
}

pub fn render_waveform(
    chord: &Chord,
    duration: Duration,
    sample_rate: u32,
    options: SynthOptions,
) -> Vec<f32> {
    let frame_count = (duration.as_secs_f64() * sample_rate as f64).ceil() as usize;
    if frame_count == 0 {
        return Vec::new();
    }

    let mut samples = vec![0.0_f32; frame_count];

    for note in chord.notes() {
        let frequency = note.frequency_hz();
        let note_seed = note_seed_for_playback(frequency, frame_count, options.seed);

        for harmonic_index in 1..=options.harmonics {
            let amplitude = (-0.6 * (harmonic_index - 1) as f32).exp();
            let phase = phase_for_harmonic(note_seed, harmonic_index);
            let phase_direction = phase_direction_for_harmonic(note_seed, harmonic_index);

            for (sample_index, sample) in samples.iter_mut().enumerate() {
                let time = sample_index as f32 / sample_rate as f32;
                let vibrato_frequency_scale = cents_to_ratio(
                    options.vibrato_depth_cents * (2.0 * PI * options.vibrato_rate_hz * time).sin(),
                );
                let harmonic_frequency =
                    frequency * vibrato_frequency_scale * harmonic_index as f32;
                let phase_mod = phase_direction
                    * options.phase_mod_depth_radians
                    * (2.0 * PI * options.phase_mod_rate_hz * time).sin();
                let amplitude_tremolo = 1.0
                    + options.amp_tremolo_depth * (2.0 * PI * options.amp_tremolo_rate_hz * time).sin();
                *sample +=
                    options.gain
                        * amplitude
                        * amplitude_tremolo
                        * (2.0 * PI * harmonic_frequency * time + phase + phase_mod).sin();
            }
        }
    }

    apply_fade(&mut samples, sample_rate);
    normalize_peak(&mut samples);

    samples
}

fn note_seed_for_playback(frequency: f32, frame_count: usize, seed: u64) -> u32 {
    let mixed = u64::from(frequency.to_bits())
        ^ (frame_count as u64).wrapping_mul(0x9E37_79B9)
        ^ seed.wrapping_mul(0xA24B_AED4_963E_E407);

    (mixed ^ (mixed >> 32)) as u32
}

fn cents_to_ratio(cents: f32) -> f32 {
    2.0_f32.powf(cents / 1200.0)
}

fn phase_for_harmonic(seed: u32, harmonic_index: u32) -> f32 {
    let mixed = seed
        .wrapping_mul(0x85EB_CA6B)
        .wrapping_add(harmonic_index.wrapping_mul(0xC2B2_AE35));
    let normalized = (mixed as f32 / u32::MAX as f32) * 2.0 * PI;
    normalized + harmonic_index as f32 * PHASE_GOLDEN_RATIO
}

fn phase_direction_for_harmonic(seed: u32, harmonic_index: u32) -> f32 {
    if harmonic_index == 1 {
        return 1.0;
    }

    let mixed = seed
        .wrapping_mul(0x27D4_EB2D)
        .wrapping_add(harmonic_index.wrapping_mul(0x1656_67B1));

    if mixed & 1 == 0 { 1.0 } else { -1.0 }
}

fn apply_fade(samples: &mut [f32], sample_rate: u32) {
    if samples.is_empty() {
        return;
    }

    let fade_samples = ((sample_rate as f32 * FADE_TIME_SECONDS).round() as usize)
        .min(samples.len() / 2)
        .max(1);

    for index in 0..fade_samples {
        let gain = index as f32 / fade_samples as f32;
        samples[index] *= gain;

        let tail_index = samples.len() - 1 - index;
        samples[tail_index] *= gain;
    }
}

fn normalize_peak(samples: &mut [f32]) {
    let peak = samples
        .iter()
        .copied()
        .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

    if peak > 0.0 {
        for sample in samples {
            *sample /= peak;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::chord::Chord;

    use super::{SynthOptions, render_waveform};

    #[test]
    fn renders_non_empty_waveform() {
        let chord = Chord::parse("A4").unwrap();
        let samples = render_waveform(
            &chord,
            Duration::from_millis(100),
            48_000,
            SynthOptions::default(),
        );
        assert!(!samples.is_empty());
    }

    #[test]
    fn normalizes_peak_to_one() {
        let chord = Chord::parse("C4+E4+G4").unwrap();
        let samples = render_waveform(
            &chord,
            Duration::from_millis(50),
            48_000,
            SynthOptions::default(),
        );
        let peak = samples
            .iter()
            .copied()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!((peak - 1.0).abs() < 0.001, "peak was {peak}");
    }

    #[test]
    fn fades_in_and_out() {
        let chord = Chord::parse("A4").unwrap();
        let samples = render_waveform(
            &chord,
            Duration::from_millis(100),
            48_000,
            SynthOptions::default(),
        );
        assert!(samples.first().unwrap().abs() < 0.001);
        assert!(samples.last().unwrap().abs() < 0.001);
    }

    #[test]
    fn seed_changes_waveform() {
        let chord = Chord::parse("A4").unwrap();
        let default_samples = render_waveform(
            &chord,
            Duration::from_millis(100),
            48_000,
            SynthOptions::default(),
        );
        let seeded_samples = render_waveform(
            &chord,
            Duration::from_millis(100),
            48_000,
            SynthOptions {
                seed: 7,
                ..SynthOptions::default()
            },
        );

        assert_ne!(default_samples, seeded_samples);
    }

    #[test]
    fn vibrato_changes_waveform() {
        let chord = Chord::parse("A4").unwrap();
        let no_vibrato_samples = render_waveform(
            &chord,
            Duration::from_millis(100),
            48_000,
            SynthOptions {
                vibrato_depth_cents: 0.0,
                ..SynthOptions::default()
            },
        );
        let vibrato_samples = render_waveform(
            &chord,
            Duration::from_millis(100),
            48_000,
            SynthOptions::default(),
        );

        assert_ne!(no_vibrato_samples, vibrato_samples);
    }
}
