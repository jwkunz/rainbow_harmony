use crate::error::RainbowError;
use crate::note::Note;

const ROOT_OCTAVE_MIDI_OFFSET: i32 = 60;
const BASS_OCTAVE_MIDI_OFFSET: i32 = 48;

#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    notes: Vec<Note>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChordQuality {
    Major,
    Minor,
    Augmented,
    Diminished,
    Suspended2,
    Suspended4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtensionKind {
    Sixth,
    DominantSeventh,
    MajorSeventh,
    Ninth,
    Eleventh,
    Thirteenth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AddedTone {
    Add2,
    Add4,
    Add9,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Alteration {
    FlatFive,
    SharpFive,
    FlatNine,
    SharpNine,
    SharpEleven,
    FlatThirteen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PitchClass {
    semitone: i32,
}

impl PitchClass {
    fn parse(input: &str) -> Result<(Self, usize), RainbowError> {
        let mut chars = input.chars();
        let letter = chars
            .next()
            .ok_or_else(|| RainbowError::usage("chord symbol cannot be empty"))?
            .to_ascii_uppercase();

        let base: i32 = match letter {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => {
                return Err(RainbowError::usage(format!(
                    "invalid chord root `{}`; expected A through G",
                    letter
                )));
            }
        };

        let mut consumed = 1;
        let accidental: i32 = match chars.next() {
            Some('#') => {
                consumed += 1;
                1
            }
            Some('b') | Some('B') => {
                consumed += 1;
                -1
            }
            _ => 0,
        };

        Ok((
            Self {
                semitone: (base + accidental).rem_euclid(12),
            },
            consumed,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChordSymbol {
    root: PitchClass,
    quality: ChordQuality,
    extension: Option<ExtensionKind>,
    added_tones: Vec<AddedTone>,
    alterations: Vec<Alteration>,
    bass: Option<PitchClass>,
}

impl Chord {
    pub fn parse(input: &str) -> Result<Self, RainbowError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(RainbowError::usage("note expression cannot be empty"));
        }

        if trimmed.contains('+') {
            return Self::parse_explicit_notes(trimmed, input);
        }

        if let Ok(note) = Note::parse(trimmed) {
            return Ok(Self { notes: vec![note] });
        }

        Self::parse_symbol(trimmed)
    }

    pub fn parse_notes(input: &str) -> Result<Self, RainbowError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(RainbowError::usage("note expression cannot be empty"));
        }

        Self::parse_explicit_notes(trimmed, input)
    }

    pub fn parse_symbol_name(input: &str) -> Result<Self, RainbowError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(RainbowError::usage("chord symbol cannot be empty"));
        }

        Self::parse_symbol(trimmed)
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    fn parse_explicit_notes(trimmed: &str, original: &str) -> Result<Self, RainbowError> {
        let notes = trimmed
            .split('+')
            .map(str::trim)
            .enumerate()
            .map(|(index, segment)| {
                if segment.is_empty() {
                    return Err(RainbowError::usage(format!(
                        "invalid chord `{}`; missing note around position {}",
                        original,
                        index + 1
                    )));
                }

                Note::parse(segment)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { notes })
    }

    fn parse_symbol(trimmed: &str) -> Result<Self, RainbowError> {
        let symbol = ChordSymbol::parse(trimmed)?;
        let notes = symbol.to_notes()?;
        Ok(Self { notes })
    }
}

impl ChordSymbol {
    fn parse(input: &str) -> Result<Self, RainbowError> {
        let (head, bass) = match input.split_once('/') {
            Some((head, bass_text)) => {
                let (bass, consumed) = PitchClass::parse(bass_text)?;
                if consumed != bass_text.len() {
                    return Err(RainbowError::usage(format!(
                        "invalid slash bass `{}` in `{}`",
                        bass_text, input
                    )));
                }
                (head, Some(bass))
            }
            None => (input, None),
        };

        let (root, root_len) = PitchClass::parse(head)?;
        let mut suffix = &head[root_len..];

        let quality = if suffix.starts_with("sus2") {
            suffix = &suffix[4..];
            ChordQuality::Suspended2
        } else if suffix.starts_with("sus4") {
            suffix = &suffix[4..];
            ChordQuality::Suspended4
        } else if suffix.starts_with("sus") {
            suffix = &suffix[3..];
            ChordQuality::Suspended4
        } else if suffix.starts_with("maj") {
            suffix = &suffix[3..];
            ChordQuality::Major
        } else if suffix.starts_with("min") {
            suffix = &suffix[3..];
            ChordQuality::Minor
        } else if suffix.starts_with('m') {
            suffix = &suffix[1..];
            ChordQuality::Minor
        } else if suffix.starts_with("aug") {
            suffix = &suffix[3..];
            ChordQuality::Augmented
        } else if suffix.starts_with('+') {
            suffix = &suffix[1..];
            ChordQuality::Augmented
        } else if suffix.starts_with("dim") {
            suffix = &suffix[3..];
            ChordQuality::Diminished
        } else if suffix.starts_with('o') {
            suffix = &suffix[1..];
            ChordQuality::Diminished
        } else {
            ChordQuality::Major
        };

        let (extension, consumed_extension) = parse_extension(suffix, quality)?;
        suffix = &suffix[consumed_extension..];

        let mut added_tones = Vec::new();
        let mut alterations = Vec::new();

        while !suffix.is_empty() {
            if let Some(rest) = suffix.strip_prefix("add9") {
                added_tones.push(AddedTone::Add9);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("add4") {
                added_tones.push(AddedTone::Add4);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("add2") {
                added_tones.push(AddedTone::Add2);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("b13") {
                alterations.push(Alteration::FlatThirteen);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("#11") {
                alterations.push(Alteration::SharpEleven);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("b9") {
                alterations.push(Alteration::FlatNine);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("#9") {
                alterations.push(Alteration::SharpNine);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("b5") {
                alterations.push(Alteration::FlatFive);
                suffix = rest;
            } else if let Some(rest) = suffix.strip_prefix("#5") {
                alterations.push(Alteration::SharpFive);
                suffix = rest;
            } else {
                return Err(RainbowError::usage(format!(
                    "unsupported chord suffix `{}` in `{}`",
                    suffix, input
                )));
            }
        }

        Ok(Self {
            root,
            quality,
            extension,
            added_tones,
            alterations,
            bass,
        })
    }

    fn to_notes(&self) -> Result<Vec<Note>, RainbowError> {
        let mut intervals = base_intervals(self.quality);
        apply_extension(&mut intervals, self.quality, self.extension);

        for added_tone in &self.added_tones {
            add_interval(&mut intervals, added_tone_interval(*added_tone));
        }

        for alteration in &self.alterations {
            apply_alteration(&mut intervals, *alteration);
        }

        intervals.sort_unstable();
        intervals.dedup();

        let mut notes = Vec::new();
        if let Some(bass) = self.bass {
            notes.push(Note::from_midi_note(BASS_OCTAVE_MIDI_OFFSET + bass.semitone)?);
        }

        for interval in intervals {
            notes.push(Note::from_midi_note(
                ROOT_OCTAVE_MIDI_OFFSET + self.root.semitone + interval,
            )?);
        }

        Ok(notes)
    }
}

fn parse_extension(
    suffix: &str,
    quality: ChordQuality,
) -> Result<(Option<ExtensionKind>, usize), RainbowError> {
    if quality == ChordQuality::Minor {
        if suffix.starts_with("Maj7") {
            return Ok((Some(ExtensionKind::MajorSeventh), 4));
        }
        if suffix.starts_with("maj7") {
            return Ok((Some(ExtensionKind::MajorSeventh), 4));
        }
        if suffix.starts_with("M7") {
            return Ok((Some(ExtensionKind::MajorSeventh), 2));
        }
    }

    let candidates = [
        ("maj7", ExtensionKind::MajorSeventh),
        ("Maj7", ExtensionKind::MajorSeventh),
        ("M7", ExtensionKind::MajorSeventh),
        ("13", ExtensionKind::Thirteenth),
        ("11", ExtensionKind::Eleventh),
        ("9", ExtensionKind::Ninth),
        ("7", ExtensionKind::DominantSeventh),
        ("6", ExtensionKind::Sixth),
    ];

    for (token, extension) in candidates {
        if suffix.starts_with(token) {
            return Ok((Some(extension), token.len()));
        }
    }

    Ok((None, 0))
}

fn base_intervals(quality: ChordQuality) -> Vec<i32> {
    match quality {
        ChordQuality::Major => vec![0, 4, 7],
        ChordQuality::Minor => vec![0, 3, 7],
        ChordQuality::Augmented => vec![0, 4, 8],
        ChordQuality::Diminished => vec![0, 3, 6],
        ChordQuality::Suspended2 => vec![0, 2, 7],
        ChordQuality::Suspended4 => vec![0, 5, 7],
    }
}

fn apply_extension(intervals: &mut Vec<i32>, quality: ChordQuality, extension: Option<ExtensionKind>) {
    let Some(extension) = extension else {
        return;
    };

    match extension {
        ExtensionKind::Sixth => add_interval(intervals, 9),
        ExtensionKind::MajorSeventh => add_interval(intervals, 11),
        ExtensionKind::DominantSeventh => {
            add_interval(
                intervals,
                if quality == ChordQuality::Diminished { 9 } else { 10 },
            );
        }
        ExtensionKind::Ninth => {
            add_interval(
                intervals,
                if quality == ChordQuality::Diminished { 9 } else { 10 },
            );
            add_interval(intervals, 14);
        }
        ExtensionKind::Eleventh => {
            add_interval(
                intervals,
                if quality == ChordQuality::Diminished { 9 } else { 10 },
            );
            add_interval(intervals, 14);
            add_interval(intervals, 17);
        }
        ExtensionKind::Thirteenth => {
            add_interval(
                intervals,
                if quality == ChordQuality::Diminished { 9 } else { 10 },
            );
            add_interval(intervals, 14);
            add_interval(intervals, 17);
            add_interval(intervals, 21);
        }
    }
}

fn added_tone_interval(added_tone: AddedTone) -> i32 {
    match added_tone {
        AddedTone::Add2 => 2,
        AddedTone::Add4 => 5,
        AddedTone::Add9 => 14,
    }
}

fn apply_alteration(intervals: &mut Vec<i32>, alteration: Alteration) {
    match alteration {
        Alteration::FlatFive => replace_interval(intervals, 7, 6),
        Alteration::SharpFive => replace_interval(intervals, 7, 8),
        Alteration::FlatNine => add_interval(intervals, 13),
        Alteration::SharpNine => add_interval(intervals, 15),
        Alteration::SharpEleven => add_interval(intervals, 18),
        Alteration::FlatThirteen => add_interval(intervals, 20),
    }
}

fn add_interval(intervals: &mut Vec<i32>, interval: i32) {
    if !intervals.contains(&interval) {
        intervals.push(interval);
    }
}

fn replace_interval(intervals: &mut Vec<i32>, from: i32, to: i32) {
    if let Some(index) = intervals.iter().position(|interval| *interval == from) {
        intervals[index] = to;
    } else {
        intervals.push(to);
    }
}

#[cfg(test)]
mod tests {
    use super::Chord;

    #[test]
    fn parses_single_note_expression() {
        let chord = Chord::parse("A4").unwrap();
        assert_eq!(chord.notes().len(), 1);
    }

    #[test]
    fn parses_chord_expression() {
        let chord = Chord::parse("C4+E4+G4").unwrap();
        assert_eq!(chord.notes().len(), 3);
    }

    #[test]
    fn preserves_duplicate_notes() {
        let chord = Chord::parse("C4+C4").unwrap();
        assert_eq!(chord.notes().len(), 2);
    }

    #[test]
    fn rejects_missing_chord_segment() {
        let error = Chord::parse("C4++E4").unwrap_err().to_string();
        assert!(error.contains("invalid chord"));
    }

    #[test]
    fn parses_major_symbol() {
        let chord = Chord::parse("C").unwrap();
        assert_eq!(chord.notes().len(), 3);
    }

    #[test]
    fn parses_minor_seventh_symbol() {
        let chord = Chord::parse("Cm7").unwrap();
        assert_eq!(chord.notes().len(), 4);
    }

    #[test]
    fn parses_major_seventh_symbol() {
        let chord = Chord::parse("Cmaj7").unwrap();
        assert_eq!(chord.notes().len(), 4);
    }

    #[test]
    fn parses_half_diminished_alias() {
        let chord = Chord::parse("Cm7b5").unwrap();
        assert_eq!(chord.notes().len(), 4);
    }

    #[test]
    fn parses_slash_chord() {
        let chord = Chord::parse("C/E").unwrap();
        assert_eq!(chord.notes().len(), 4);
    }

    #[test]
    fn parses_suspended_and_added_tone_symbols() {
        let sus = Chord::parse("Csus4").unwrap();
        let add = Chord::parse("Cadd9").unwrap();

        assert_eq!(sus.notes().len(), 3);
        assert_eq!(add.notes().len(), 4);
    }

    #[test]
    fn rejects_unsupported_suffix() {
        let error = Chord::parse("Cfoobar").unwrap_err().to_string();
        assert!(error.contains("unsupported chord suffix"));
    }

    #[test]
    fn plus_requires_explicit_notes() {
        let error = Chord::parse("C+E+G").unwrap_err().to_string();
        assert!(error.contains("missing octave"));
    }
}
