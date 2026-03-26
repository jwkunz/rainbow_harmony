use crate::chord::Chord;
use crate::progression::PlaybackEvent;

#[derive(Debug, Clone, PartialEq)]
pub struct NoteDescription {
    pub name: String,
    pub frequency_hz: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EventDescription {
    pub notes: Vec<NoteDescription>,
}

pub fn describe_chord(chord: &Chord) -> EventDescription {
    EventDescription {
        notes: chord
            .notes()
            .iter()
            .map(|note| NoteDescription {
                name: note.to_string(),
                frequency_hz: note.frequency_hz(),
            })
            .collect(),
    }
}

pub fn describe_event(event: &PlaybackEvent) -> EventDescription {
    describe_chord(&event.chord)
}

pub fn format_event_description(event: &PlaybackEvent) -> String {
    let description = describe_event(event);
    let note_names = description
        .notes
        .iter()
        .map(|note| note.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let frequencies = description
        .notes
        .iter()
        .map(|note| format!("{:.2}", note.frequency_hz))
        .collect::<Vec<_>>()
        .join(", ");

    format!("notes: {note_names}\nfrequencies_hz: {frequencies}")
}
