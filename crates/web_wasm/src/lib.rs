use wasm_bindgen::prelude::*;

use rainbow_harmony_core::error::RainbowError;
use rainbow_harmony_core::progression::{
    ProgressionMode, ProgressionPlaybackOptions, parse_progression_text, parse_single_event_args,
    render_progression_waveform,
};
use rainbow_harmony_core::synth::render_waveform;

#[wasm_bindgen]
pub struct RenderResult {
    sample_rate: u32,
    samples: Vec<f32>,
}

#[wasm_bindgen]
impl RenderResult {
    #[wasm_bindgen(getter)]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn samples(&self) -> Vec<f32> {
        self.samples.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn frame_count(&self) -> usize {
        self.samples.len()
    }

    #[wasm_bindgen(getter)]
    pub fn duration_seconds(&self) -> f64 {
        self.samples.len() as f64 / self.sample_rate as f64
    }
}

#[wasm_bindgen]
pub fn render_event_args(args_text: &str, sample_rate: u32) -> Result<RenderResult, JsValue> {
    let args = tokenize_args(args_text);
    let event = parse_single_event_args(&args).map_err(js_error)?;
    let samples = render_waveform(
        &event.chord,
        event.duration,
        sample_rate,
        event.synth_options,
    );

    Ok(RenderResult {
        sample_rate,
        samples,
    })
}

#[wasm_bindgen]
pub fn render_progression(
    progression_text: &str,
    sample_rate: u32,
    mode: &str,
    gap_ms: u32,
    legato_ms: u32,
) -> Result<RenderResult, JsValue> {
    let events = parse_progression_text(progression_text).map_err(js_error)?;
    let samples = render_progression_waveform(
        &events,
        sample_rate,
        ProgressionPlaybackOptions {
            mode: parse_progression_mode(mode)?,
            gap_ms,
            legato_ms,
        },
    );

    Ok(RenderResult {
        sample_rate,
        samples,
    })
}

fn parse_progression_mode(input: &str) -> Result<ProgressionMode, JsValue> {
    match input {
        "stitched" => Ok(ProgressionMode::Stitched),
        "separate" => Ok(ProgressionMode::Separate),
        _ => Err(JsValue::from_str(
            "invalid progression mode; expected stitched or separate",
        )),
    }
}

fn tokenize_args(args_text: &str) -> Vec<String> {
    args_text.split_whitespace().map(str::to_string).collect()
}

fn js_error(error: RainbowError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{render_event_args, render_progression};

    #[test]
    fn renders_single_event_from_flag_text() {
        let result = render_event_args("--notes A4 --seconds 0.1", 48_000).unwrap();
        assert!(!result.samples().is_empty());
        assert_eq!(result.sample_rate(), 48_000);
    }

    #[test]
    fn renders_progression_from_text() {
        let result = render_progression(
            "--chord Cmaj7 --beats 1 => --chord Fmaj7",
            48_000,
            "stitched",
            0,
            0,
        )
        .unwrap();

        assert!(!result.samples().is_empty());
        assert_eq!(result.sample_rate(), 48_000);
    }
}
