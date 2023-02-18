// Based on: https://github.com/derekdreery/nom-midi-rs/
#[cfg(test)]
mod new;
mod types;
mod utils;
pub use self::types::Note;
use nom::number::streaming::be_u8;
use nom::{
    error::{make_error, ErrorKind},
    Err, IResult,
};

pub trait ToMidi {
    fn to_midi(&self) -> Vec<u8> {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoteEvent {
    pub channel: u8,
    pub note: u8,
    pub velocity: u8,
}

impl ToMidi for NoteEvent {
    fn to_midi(&self) -> Vec<u8> {
        vec![0x80 + self.channel, self.note.into(), self.velocity]
    }
}

impl NoteEvent {
    pub fn get_note_off(&self) -> NoteEvent {
        NoteEvent {
            velocity: 0,
            ..*self
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PolyphonicPressure {
    channel: u8,
    note: u8,
    pressure: u8,
}

impl ToMidi for PolyphonicPressure {
    fn to_midi(&self) -> Vec<u8> {
        vec![0xA0 + self.channel, self.note.into(), self.pressure]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Controller {
    pub channel: u8,
    pub controller: u8,
    pub value: u8,
}

impl ToMidi for Controller {
    fn to_midi(&self) -> Vec<u8> {
        vec![0xB0 + self.channel, self.controller, self.value]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ProgramChange {
    channel: u8,
    program: u8,
}

impl ToMidi for ProgramChange {
    fn to_midi(&self) -> Vec<u8> {
        vec![0xC0 + self.channel, self.program]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChannelPressure {
    channel: u8,
    pressure: u8,
}

impl ToMidi for ChannelPressure {
    fn to_midi(&self) -> Vec<u8> {
        vec![0xD0 + self.channel, self.pressure]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PitchBend {
    channel: u8,
    lsb: u8,
    msb: u8,
}

impl ToMidi for PitchBend {
    fn to_midi(&self) -> Vec<u8> {
        vec![0xE0 + self.channel, self.lsb, self.msb]
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MIDIRouterEvent {
    pub device: String,
    pub event: MIDIEvent,
}

impl std::fmt::Display for MIDIRouterEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {:?}", self.device, self.event)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MIDIEvent {
    NoteOff(NoteEvent),
    NoteOn(NoteEvent),
    PolyphonicPressure(PolyphonicPressure),
    Controller(Controller),
    ChannelPressure(ChannelPressure),
    ProgramChange(ProgramChange),
    PitchBend(PitchBend),
}

impl MIDIEvent {
    pub fn get_channel(&self) -> u8 {
        match self {
            Self::NoteOn(note) => note.channel,
            Self::NoteOff(v) => v.channel,
            Self::PolyphonicPressure(v) => v.channel,
            Self::Controller(v) => v.channel,
            Self::ChannelPressure(v) => v.channel,
            Self::ProgramChange(v) => v.channel,
            Self::PitchBend(v) => v.channel,
        }
    }

    pub fn set_channel(&mut self, new_channel: u8) {
        match self {
            Self::NoteOn(note) => note.channel = new_channel,
            Self::NoteOff(v) => v.channel = new_channel,
            Self::PolyphonicPressure(v) => v.channel = new_channel,
            Self::Controller(v) => v.channel = new_channel,
            Self::ChannelPressure(v) => v.channel = new_channel,
            Self::ProgramChange(v) => v.channel = new_channel,
            Self::PitchBend(v) => v.channel = new_channel,
        };
    }
}

impl TryFrom<&[u8]> for MIDIEvent {
    type Error = ();
    fn try_from(i: &[u8]) -> Result<Self, Self::Error> {
        if let Ok((_i, result)) = parse_midi_event(i) {
            Ok(result)
        } else {
            Err(())
        }
    }
}

impl ToMidi for MIDIEvent {
    fn to_midi(&self) -> Vec<u8> {
        match self {
            MIDIEvent::NoteOff(v) => v.to_midi(),
            MIDIEvent::NoteOn(v) => v.to_midi(),
            MIDIEvent::PolyphonicPressure(v) => v.to_midi(),
            MIDIEvent::Controller(v) => v.to_midi(),
            MIDIEvent::ChannelPressure(v) => v.to_midi(),
            MIDIEvent::ProgramChange(v) => v.to_midi(),
            MIDIEvent::PitchBend(v) => v.to_midi(),
        }
    }
}

pub fn parse_midi_event(i: &[u8]) -> IResult<&[u8], MIDIEvent> {
    let (i, code_chan) = be_u8(i)?;

    let event_type = code_chan >> 4;
    let channel = code_chan & 0x0F;

    let result = match event_type {
        0x8 => {
            let (i, note_code) = utils::be_u7(i)?;
            let (i, velocity) = utils::be_u7(i)?;

            MIDIEvent::NoteOff(NoteEvent {
                channel,
                note: note_code.into(),
                velocity,
            })
        }

        0x9 => {
            let (i, note_code) = utils::be_u7(i)?;
            let (i, velocity) = utils::be_u7(i)?;

            MIDIEvent::NoteOn(NoteEvent {
                channel,
                note: note_code.into(),
                velocity,
            })
        }

        0xA => {
            let (i, note_code) = utils::be_u7(i)?;
            let (i, pressure) = utils::be_u7(i)?;

            MIDIEvent::PolyphonicPressure(PolyphonicPressure {
                channel,
                note: note_code.into(),
                pressure,
            })
        }

        0xB => {
            let (i, controller) = be_u8(i)?;
            let (i, value) = utils::be_u7(i)?;

            MIDIEvent::Controller(Controller {
                channel,
                controller,
                value,
            })
        }

        0xC => {
            let (i, program) = utils::be_u7(i)?;

            MIDIEvent::ProgramChange(ProgramChange { channel, program })
        }

        0xD => {
            let (i, pressure) = utils::be_u7(i)?;

            MIDIEvent::ChannelPressure(ChannelPressure { channel, pressure })
        }

        0xE => {
            let (i, lsb) = utils::be_u7(i)?;
            let (i, msb) = utils::be_u7(i)?;

            MIDIEvent::PitchBend(PitchBend { channel, lsb, msb })
        }

        0xFA => {
            dbg!("--------------");
            dbg!(i);

            return Err(Err::Error(make_error(i, ErrorKind::Digit)));
        }

        _ => return Err(Err::Error(make_error(i, ErrorKind::Digit))),
    };

    Ok((i, result))
}
