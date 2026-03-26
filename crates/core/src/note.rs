use std::fmt::{self, Display, Formatter};

use crate::error::RainbowError;

const MIN_OCTAVE: u8 = 0;
const MAX_OCTAVE: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Note {
    letter: char,
    accidental: Accidental,
    octave: u8,
}

impl Note {
    pub fn from_parts(
        letter: char,
        accidental: Accidental,
        octave: u8,
    ) -> Result<Self, RainbowError> {
        let letter = letter.to_ascii_uppercase();
        if !matches!(letter, 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G') {
            return Err(RainbowError::usage(format!(
                "invalid note letter `{}`; expected A through G",
                letter
            )));
        }

        if !(MIN_OCTAVE..=MAX_OCTAVE).contains(&octave) {
            return Err(RainbowError::usage(format!(
                "octave out of range; expected {} through {}",
                MIN_OCTAVE, MAX_OCTAVE
            )));
        }

        Ok(Self {
            letter,
            accidental,
            octave,
        })
    }

    pub fn parse(input: &str) -> Result<Self, RainbowError> {
        let normalized = input.trim();
        if normalized.is_empty() {
            return Err(RainbowError::usage("note cannot be empty"));
        }

        let mut chars = normalized.chars();
        let letter = chars
            .next()
            .ok_or_else(|| RainbowError::usage("note cannot be empty"))?
            .to_ascii_uppercase();

        if !matches!(letter, 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G') {
            return Err(RainbowError::usage(format!(
                "invalid note letter `{}` in `{}`; expected A through G",
                letter, input
            )));
        }

        let remainder = chars.as_str();
        let (accidental, octave_text) = match remainder.chars().next() {
            Some('#') => (Accidental::Sharp, &remainder[1..]),
            Some('b') | Some('B') => (Accidental::Flat, &remainder[1..]),
            Some(_) => (Accidental::Natural, remainder),
            None => {
                return Err(RainbowError::usage(format!(
                    "missing octave in note `{}`; expected format like C4 or Bb3",
                    input
                )));
            }
        };

        if octave_text.is_empty() {
            return Err(RainbowError::usage(format!(
                "missing octave in note `{}`; expected a value from {} to {}",
                input, MIN_OCTAVE, MAX_OCTAVE
            )));
        }

        let octave: u8 = octave_text.parse().map_err(|_| {
            RainbowError::usage(format!(
                "invalid octave in note `{}`; expected a number from {} to {}",
                input, MIN_OCTAVE, MAX_OCTAVE
            ))
        })?;

        if !(MIN_OCTAVE..=MAX_OCTAVE).contains(&octave) {
            return Err(RainbowError::usage(format!(
                "octave out of range in note `{}`; expected {} through {}",
                input, MIN_OCTAVE, MAX_OCTAVE
            )));
        }

        Self::from_parts(letter, accidental, octave)
    }

    pub fn frequency_hz(self) -> f32 {
        let semitone = match self.letter {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => unreachable!("validated note letter"),
        };

        let accidental_offset = match self.accidental {
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::Flat => -1,
        };

        let midi_note = ((self.octave as i32 + 1) * 12) + semitone + accidental_offset;
        let semitones_from_a4 = midi_note - 69;

        440.0_f32 * 2.0_f32.powf(semitones_from_a4 as f32 / 12.0)
    }

    pub fn from_midi_note(midi_note: i32) -> Result<Self, RainbowError> {
        let octave = (midi_note / 12) - 1;
        if !(i32::from(MIN_OCTAVE)..=i32::from(MAX_OCTAVE)).contains(&octave) {
            return Err(RainbowError::usage(format!(
                "generated note out of supported octave range; expected {} through {}",
                MIN_OCTAVE, MAX_OCTAVE
            )));
        }

        let pitch_class = midi_note.rem_euclid(12);
        let (letter, accidental) = match pitch_class {
            0 => ('C', Accidental::Natural),
            1 => ('C', Accidental::Sharp),
            2 => ('D', Accidental::Natural),
            3 => ('D', Accidental::Sharp),
            4 => ('E', Accidental::Natural),
            5 => ('F', Accidental::Natural),
            6 => ('F', Accidental::Sharp),
            7 => ('G', Accidental::Natural),
            8 => ('G', Accidental::Sharp),
            9 => ('A', Accidental::Natural),
            10 => ('A', Accidental::Sharp),
            11 => ('B', Accidental::Natural),
            _ => unreachable!("pitch class is modulo 12"),
        };

        Self::from_parts(letter, accidental, octave as u8)
    }
}

impl Display for Note {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let accidental = match self.accidental {
            Accidental::Natural => "",
            Accidental::Sharp => "#",
            Accidental::Flat => "b",
        };

        write!(f, "{}{}{}", self.letter, accidental, self.octave)
    }
}

#[cfg(test)]
mod tests {
    use super::Note;

    fn assert_frequency_close(actual: f32, expected: f32) {
        let delta = (actual - expected).abs();
        assert!(
            delta < 0.05,
            "expected frequency near {expected}, got {actual} (delta {delta})"
        );
    }

    #[test]
    fn parses_natural_note() {
        let note = Note::parse("A4").unwrap();
        assert_frequency_close(note.frequency_hz(), 440.0);
    }

    #[test]
    fn parses_flat_note() {
        let note = Note::parse("Bb3").unwrap();
        assert_frequency_close(note.frequency_hz(), 233.08188);
    }

    #[test]
    fn parses_lowercase_input() {
        let note = Note::parse("c#4").unwrap();
        assert_frequency_close(note.frequency_hz(), 277.18262);
    }

    #[test]
    fn rejects_invalid_note_letter() {
        let error = Note::parse("H2").unwrap_err().to_string();
        assert!(error.contains("invalid note letter"));
    }

    #[test]
    fn rejects_missing_octave() {
        let error = Note::parse("Cb").unwrap_err().to_string();
        assert!(error.contains("missing octave"));
    }

    #[test]
    fn rejects_out_of_range_octave() {
        let error = Note::parse("A9").unwrap_err().to_string();
        assert!(error.contains("octave out of range"));
    }

    #[test]
    fn builds_note_from_midi() {
        let note = Note::from_midi_note(60).unwrap();
        assert_frequency_close(note.frequency_hz(), 261.62558);
    }
}
