#[cfg(test)]
mod tests {
  use crate::midi_event::{utils::compare_u8_slices, MidiEvent, Note};
  use std::cmp;

  #[test]
  fn from_midi_1() {
    let message = vec![128, 70, 0]; // Ch0, NoteOff, As4, 0

    assert_eq!(
      MidiEvent::from_midi(&message).unwrap(),
      MidiEvent::NoteOff {
        channel: 0,
        note: Note::As4,
        velocity: 0
      }
    );
  }

  #[test]
  fn from_midi_2() {
    let message = vec![144, 0, 43]; // Ch0, NoteOff, As4, 0

    assert_eq!(
      MidiEvent::from_midi(&message).unwrap(),
      MidiEvent::NoteOn {
        channel: 0,
        note: Note::C1n,
        velocity: 43
      }
    );
  }

  #[test]
  fn to_midi() {
    let cases = [
      vec![144, 70, 43],  // Ch0, NoteOff, As4, 0
      vec![128, 70, 0],   // Ch0, NoteOff, As4, 0
      vec![176, 59, 127], // Ch0, Controller, 59, 127
    ];

    cases.iter().for_each(|message| {
      let parsed = MidiEvent::from_midi(&message).unwrap();

      assert_eq!(
        compare_u8_slices(parsed.to_midi().unwrap().as_ref(), &message),
        cmp::Ordering::Equal
      );
    })
  }
}
