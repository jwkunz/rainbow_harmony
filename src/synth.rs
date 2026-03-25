use std::f32::consts::PI;
use std::time::Duration;

use crate::chord::Chord;

const HARMONIC_COUNT: u32 = 33;
const FADE_TIME_SECONDS: f32 = 0.01;
const PHASE_GOLDEN_RATIO: f32 = 2.399_963_1;

pub fn render_waveform(chord: &Chord, duration: Duration, sample_rate: u32) -> Vec<f32> {
    let frame_count = (duration.as_secs_f64() * sample_rate as f64).ceil() as usize;
    if frame_count == 0 {
        return Vec::new();
    }

    let mut samples = vec![0.0_f32; frame_count];

    for note in chord.notes() {
        let frequency = note.frequency_hz();
        let note_seed = frequency.to_bits() ^ (frame_count as u32).wrapping_mul(0x9E37_79B9);

        for harmonic_index in 1..=HARMONIC_COUNT {
            let harmonic_frequency = frequency * harmonic_index as f32;
            let amplitude = (-0.6 * (harmonic_index - 1) as f32).exp();
            let phase = phase_for_harmonic(note_seed, harmonic_index);

            for (sample_index, sample) in samples.iter_mut().enumerate() {
                let time = sample_index as f32 / sample_rate as f32;
                *sample += amplitude * (2.0 * PI * harmonic_frequency * time + phase).sin();
            }
        }
    }

    apply_fade(&mut samples, sample_rate);
    normalize_peak(&mut samples);

    samples
}

fn phase_for_harmonic(seed: u32, harmonic_index: u32) -> f32 {
    let mixed = seed
        .wrapping_mul(0x85EB_CA6B)
        .wrapping_add(harmonic_index.wrapping_mul(0xC2B2_AE35));
    let normalized = (mixed as f32 / u32::MAX as f32) * 2.0 * PI;
    normalized + harmonic_index as f32 * PHASE_GOLDEN_RATIO
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

    use super::render_waveform;

    #[test]
    fn renders_non_empty_waveform() {
        let chord = Chord::parse("A4").unwrap();
        let samples = render_waveform(&chord, Duration::from_millis(100), 48_000);
        assert!(!samples.is_empty());
    }

    #[test]
    fn normalizes_peak_to_one() {
        let chord = Chord::parse("C4+E4+G4").unwrap();
        let samples = render_waveform(&chord, Duration::from_millis(50), 48_000);
        let peak = samples
            .iter()
            .copied()
            .fold(0.0_f32, |acc, sample| acc.max(sample.abs()));

        assert!((peak - 1.0).abs() < 0.001, "peak was {peak}");
    }

    #[test]
    fn fades_in_and_out() {
        let chord = Chord::parse("A4").unwrap();
        let samples = render_waveform(&chord, Duration::from_millis(100), 48_000);
        assert!(samples.first().unwrap().abs() < 0.001);
        assert!(samples.last().unwrap().abs() < 0.001);
    }
}
