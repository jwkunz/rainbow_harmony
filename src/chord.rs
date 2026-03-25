use crate::error::RainbowError;
use crate::note::Note;

#[derive(Debug, Clone, PartialEq)]
pub struct Chord {
    notes: Vec<Note>,
}

impl Chord {
    pub fn parse(input: &str) -> Result<Self, RainbowError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(RainbowError::usage("note expression cannot be empty"));
        }

        let notes = trimmed
            .split('+')
            .map(str::trim)
            .enumerate()
            .map(|(index, segment)| {
                if segment.is_empty() {
                    return Err(RainbowError::usage(format!(
                        "invalid chord `{}`; missing note around position {}",
                        input,
                        index + 1
                    )));
                }

                Note::parse(segment)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { notes })
    }

    pub fn notes(&self) -> &[Note] {
        &self.notes
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
}
