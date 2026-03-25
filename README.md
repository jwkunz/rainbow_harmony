# rainbow_harmony

`rainbow_harmony` is a cross-platform Rust CLI that plays a musical note or chord through your default audio output device.

You give it:

- a note or chord expression such as `A4` or `C4+E4+G4` via `--notes`
- either a duration in beats via `--beats` with optional `--bpm`
- or an absolute duration via `--seconds`
- optional synthesis controls such as `--gain`, `--harmonics`, phase modulation, vibrato, tremolo, white noise, and `--seed`

It generates the waveform in-process, adds a short fade-in and fade-out to reduce clicks, and blocks until playback finishes.

## Status

The project currently targets local builds. GitHub release packaging for macOS, Linux, and Windows is planned for a later step.

## Features

- single-note playback
- chord playback with `+`
- natural notes, sharps, and flats
- octave range `0` through `8`
- lowercase note input accepted and normalized
- positive integer beats
- optional positive integer BPM, defaulting to `120`
- additive synthesis using a configurable harmonic count, defaulting to 33 partials
- exponentially decreasing harmonic intensity with configurable falloff
- fresh harmonic phase variation on each playback
- phase modulation enabled by default at 1000 Hz with depth `0.05`
- overtones independently choose random positive or negative phase sweep directions
- amplitude tremolo defaulting to 3 Hz with depth `0.05`
- default vibrato at 5 Hz with 0.25 cents of depth
- white noise mixed in at a default depth of 2%
- reproducible seeded phase variation
- final waveform peak-normalized to `1.0`

## Command Syntax

```bash
rainbow_harmony --notes <notes> [--beats <beats> [--bpm <bpm>] | --seconds <seconds>] [options]
```

Examples:

```bash
rainbow_harmony --notes A4 --beats 2
rainbow_harmony --notes c4 --beats 2
rainbow_harmony --notes Bb3 --beats 4
rainbow_harmony --notes C4+E4+G4 --beats 2
rainbow_harmony --notes A3+C#4+E4 --beats 4 --bpm 90
rainbow_harmony --notes A4 --seconds 1.5
rainbow_harmony --notes A4
rainbow_harmony --notes A4 --seconds 2.0 --harmonics 12 --amp-tremolo-depth 0.2
```

Help:

```bash
rainbow_harmony --help
```

## Note Format

Each note is written as:

```text
<letter><accidental?><octave>
```

Valid letters:

- `A`
- `B`
- `C`
- `D`
- `E`
- `F`
- `G`

Accidentals:

- sharp: `#`
- flat: `b`

Valid examples:

- `A4`
- `C#4`
- `Bb3`
- `g5`
- `f#2`

Invalid examples:

- `H4`
- `C`
- `A9`
- `C##4`

## Chords

Join notes with `+` to play them simultaneously.

Examples:

```bash
rainbow_harmony --notes C4+E4 --beats 2
rainbow_harmony --notes C4+E4+G4 --beats 2
rainbow_harmony --notes C4+C4 --beats 1
```

Duplicate notes are allowed.

## Duration

- `--notes` is required
- use at most one of `--beats` or `--seconds`
- `--beats` must be a positive integer
- `--seconds` must be a positive number and defaults to `2.0` when omitted
- `--bpm` is optional, defaults to `120`, and may only be used with `--beats`

## Synthesis Flags

- `--gain <gain>` sets output gain before normalization, default `1.0`
- `--harmonics <count>` sets the number of harmonics including the fundamental, default `33`
- `--harmonic-decay <rate>` sets the exponential harmonic falloff rate, default `0.8`
- `--phase-mod-rate <hz>` sets phase modulation rate, default `1000.0`
- `--phase-mod-depth <radians>` sets phase modulation depth, default `0.05`
- `--amp-tremolo-rate <hz>` sets amplitude tremolo rate, default `3.0`
- `--amp-tremolo-depth <depth>` sets amplitude tremolo depth from `0.0` to `1.0`, default `0.05`
- `--vibrato-rate <hz>` sets vibrato rate, default `5.0`
- `--vibrato-depth-cents <cents>` sets vibrato depth in cents, default `0.25`
- `--noise-depth <depth>` sets white noise depth from `0.0` to `1.0`, default `0.02`
- `--seed <integer>` sets a deterministic seed for reproducible phase variation

Example:

```bash
rainbow_harmony \
  --notes C4+E4+G4 \
  --beats 4 \
  --bpm 72 \
  --gain 0.8 \
  --harmonics 16 \
  --harmonic-decay 0.4 \
  --phase-mod-rate 1000 \
  --phase-mod-depth 0.05 \
  --amp-tremolo-rate 3 \
  --amp-tremolo-depth 0.2 \
  --vibrato-rate 5 \
  --vibrato-depth-cents 0.25 \
  --noise-depth 0.03 \
  --seed 42
```

Duration is computed from beats and BPM:

```text
seconds = beats * 60 / bpm
```

Examples:

- `2` beats at `120` BPM = `1.0` second
- `4` beats at `120` BPM = `2.0` seconds
- `1` beat at `90` BPM = `0.666...` seconds

## Installation

### From Source

Prerequisites:

- Rust toolchain installed through `rustup`
- a working default audio output device
- platform audio development libraries if your OS requires them for native builds

Clone and build:

```bash
git clone <your-repo-url>
cd rainbow_harmony
cargo build --release
```

Run from the release binary:

```bash
./target/release/rainbow_harmony --notes A4 --beats 2
```

### Linux Notes

On Linux, native audio backends may require system packages. On Debian/Ubuntu-family systems, ALSA development headers are commonly needed:

```bash
sudo apt install libasound2-dev
```

If your distribution uses different package names, install the equivalent ALSA development package.

### Future GitHub Release Installation

Once release automation exists, the intended installation flow from a GitHub release page will be:

1. Open the repository's Releases page.
2. Download the archive for your platform.
3. Extract the archive.
4. Run the included `rainbow_harmony` binary from a terminal.

Planned release artifact shapes:

- Linux: `rainbow_harmony-linux-x86_64.tar.gz`
- macOS Intel: `rainbow_harmony-macos-x86_64.tar.gz`
- macOS Apple Silicon: `rainbow_harmony-macos-aarch64.tar.gz`
- Windows: `rainbow_harmony-windows-x86_64.zip`

Expected usage after extracting a release:

Linux/macOS:

```bash
./rainbow_harmony --notes C4+E4+G4 --beats 2
```

Windows PowerShell:

```powershell
.\rainbow_harmony.exe --notes C4+E4+G4 --beats 2
```

If you want the command available globally, move the binary into a directory that is already on your `PATH`, or add the extraction directory to your `PATH`.

## Usage Examples

Play middle C for the default two-second duration:

```bash
rainbow_harmony --notes C4
```

Play A major as a chord for three beats:

```bash
rainbow_harmony --notes A3+C#4+E4 --beats 3
```

Play a slower chord at 72 BPM:

```bash
rainbow_harmony --notes D4+F#4+A4 --beats 4 --bpm 72
```

Play for an absolute duration in seconds:

```bash
rainbow_harmony --notes A4 --seconds 1.25
```

## Error Behavior

Invalid input prints:

- a targeted error message
- usage information

Examples of invalid input:

- unknown note letter
- missing octave
- octave out of range
- malformed chord syntax
- zero or non-integer beats
- zero or non-integer BPM

## Audio Behavior

`rainbow_harmony` currently:

- uses the system default output audio device
- generates audio samples in-process
- renders the configured harmonic count for each note
- uses exponentially decreasing harmonic amplitudes with configurable falloff
- varies harmonic phase on each playback
- modulates oscillator phase with configurable rate and depth
- applies true vibrato by modulating pitch with configurable rate and cents depth
- assigns each overtone its own per-playback positive or negative phase modulation direction
- applies amplitude modulation with configurable rate and depth
- mixes in configurable white noise
- supports deterministic seeded phase variation for repeatable results
- applies a short fade-in and fade-out
- normalizes the final mixed waveform to a peak amplitude of `1.0`

The command blocks until the requested note or chord has finished playing.

## Development

Run tests:

```bash
cargo test
```

Format the code:

```bash
cargo fmt
```

Run the CLI directly during development:

```bash
cargo run -- --notes C4+E4+G4 --beats 2
```

## Roadmap

Planned follow-up work:

- GitHub Actions workflow for tagged releases on `main`
- automated Windows, macOS, and Linux release builds
- packaged downloadable release archives
- additional test coverage around CLI error output
- additional synthesis and export flags in the expanded flag-based CLI
- optional device selection and waveform controls
