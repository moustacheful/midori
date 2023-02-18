#[cfg(test)]
mod tests {
    use crate::midi_event::{utils::compare_u8_slices, MIDIEvent, Note, NoteEvent, ToMidi};
    use std::cmp;

    #[test]
    fn from_midi_1() {
        let message: &[u8] = &[128, 70, 0]; // Ch0, NoteOff, As4, 0

        assert_eq!(
            MIDIEvent::try_from(message).unwrap(),
            MIDIEvent::NoteOff(NoteEvent {
                channel: 0,
                note: Note::As4.into(),
                velocity: 0
            })
        );
    }

    #[test]
    fn from_midi_2() {
        let message: &[u8] = &[144, 0, 43]; // Ch0, NoteOff, As4, 0

        assert_eq!(
            MIDIEvent::try_from(message).unwrap(),
            MIDIEvent::NoteOn(NoteEvent {
                channel: 0,
                note: Note::C1n.into(),
                velocity: 43
            })
        );
    }

    #[test]
    fn to_midi() {
        let cases: [&[u8]; 3] = [
            &[144, 70, 43],  // Ch0, NoteOff, As4, 0
            &[128, 70, 0],   // Ch0, NoteOff, As4, 0
            &[176, 59, 127], // Ch0, Controller, 59, 127
        ];

        cases.into_iter().for_each(|message| {
            let parsed = MIDIEvent::try_from(message).unwrap();

            assert_eq!(
                compare_u8_slices(parsed.to_midi().as_ref(), message),
                cmp::Ordering::Equal
            );
        })
    }
}
