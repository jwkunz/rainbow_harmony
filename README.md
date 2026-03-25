# rainbow_harmony

`rainbow_harmony` is a cross-platform Rust CLI that plays a musical note or chord through your default audio output device.

You give it:

- a note or chord expression such as `A4` or `C4+E4+G4`
- a duration in beats
- an optional tempo in beats per minute

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
- additive synthesis using the fundamental plus 32 harmonics
- exponentially decreasing harmonic intensity
- fresh harmonic phase variation on each playback
- final waveform peak-normalized to `1.0`

## Command Syntax

```bash
rainbow_harmony <notes> <beats> [bpm]
```

Examples:

```bash
rainbow_harmony A4 2
rainbow_harmony c4 2
rainbow_harmony Bb3 4
rainbow_harmony C4+E4+G4 2
rainbow_harmony A3+C#4+E4 4 90
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
rainbow_harmony C4+E4 2
rainbow_harmony C4+E4+G4 2
rainbow_harmony C4+C4 1
```

Duplicate notes are allowed.

## Beats And BPM

- `beats` is required and must be a positive integer
- `bpm` is optional and must be a positive integer
- default BPM is `120`

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
./target/release/rainbow_harmony A4 2
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
./rainbow_harmony C4+E4+G4 2
```

Windows PowerShell:

```powershell
.\rainbow_harmony.exe C4+E4+G4 2
```

If you want the command available globally, move the binary into a directory that is already on your `PATH`, or add the extraction directory to your `PATH`.

## Usage Examples

Play middle C for two beats at the default tempo:

```bash
rainbow_harmony C4 2
```

Play A major as a chord for three beats:

```bash
rainbow_harmony A3+C#4+E4 3
```

Play a slower chord at 72 BPM:

```bash
rainbow_harmony D4+F#4+A4 4 72
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
- renders the fundamental and the next 32 harmonics for each note
- uses exponentially decreasing harmonic amplitudes
- varies harmonic phase on each playback
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
cargo run -- C4+E4+G4 2
```

## Roadmap

Planned follow-up work:

- GitHub Actions workflow for tagged releases on `main`
- automated Windows, macOS, and Linux release builds
- packaged downloadable release archives
- additional test coverage around CLI error output
- optional device selection and waveform controls
