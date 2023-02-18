impl MidiEvent {
    pub fn from_midi(i: &[u8]) -> Result<Self, Box<dyn Error>> {
        let (_i, result) = parse_midi_event(i).expect("Could not parse MIDI message");

        Ok(result)
    }

    pub fn to_midi(self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut result: Vec<u8> = vec![];

        match self {
            MidiEvent::NoteOff {
                channel,
                note,
                velocity,
            } => {
                result.push(0x80 + channel);
                result.push(note.into());
                result.push(velocity);
            }

            MidiEvent::NoteOn {
                channel,
                note,
                velocity,
            } => {
                result.push(0x90 + channel);
                result.push(note.into());
                result.push(velocity);
            }

            MidiEvent::PolyphonicPressure {
                channel,
                note,
                pressure,
            } => {
                result.push(0xA0 + channel);
                result.push(note.into());
                result.push(pressure);
            }

            MidiEvent::Controller {
                channel,
                controller,
                value,
            } => {
                result.push(0xB0 + channel);
                result.push(controller);
                result.push(value);
            }

            // UNTESTED
            MidiEvent::ProgramChange { channel, program } => {
                result.push(0xC0 + channel);
                result.push(program);
            }

            MidiEvent::ChannelPressure { channel, pressure } => {
                result.push(0xD0 + channel);
                result.push(pressure);
            }

            MidiEvent::PitchBend { channel, lsb, msb } => {
                result.push(0xE0 + channel);
                result.push(lsb);
                result.push(msb);
            }
        }

        Ok(result)
    }

    pub fn get_channel(&self) -> &u8 {
        match self {
            MidiEvent::NoteOff { channel, .. } => channel,
            MidiEvent::NoteOn { channel, .. } => channel,
            MidiEvent::PolyphonicPressure { channel, .. } => channel,
            MidiEvent::Controller { channel, .. } => channel,
            MidiEvent::ProgramChange { channel, .. } => channel,
            MidiEvent::ChannelPressure { channel, .. } => channel,
            MidiEvent::PitchBend { channel, .. } => channel,
        }
    }

    pub fn get_note_off(&self) -> Option<MidiEvent> {
        match *self {
            MidiEvent::NoteOn { channel, note, .. } => Some(MidiEvent::NoteOff {
                channel,
                note,
                velocity: 0,
            }),
            _ => None,
        }
    }
    pub fn set_channel(self, channel: u8) -> MidiEvent {
        match self {
            MidiEvent::NoteOff { note, velocity, .. } => MidiEvent::NoteOff {
                channel,
                note,
                velocity,
            },
            MidiEvent::NoteOn { note, velocity, .. } => MidiEvent::NoteOn {
                channel,
                note,
                velocity,
            },
            MidiEvent::PolyphonicPressure { note, pressure, .. } => MidiEvent::PolyphonicPressure {
                channel,
                note,
                pressure,
            },
            MidiEvent::Controller {
                controller, value, ..
            } => MidiEvent::Controller {
                channel,
                controller,
                value,
            },
            MidiEvent::ProgramChange { program, .. } => {
                MidiEvent::ProgramChange { channel, program }
            }
            MidiEvent::ChannelPressure { pressure, .. } => {
                MidiEvent::ChannelPressure { channel, pressure }
            }
            MidiEvent::PitchBend { lsb, msb, .. } => MidiEvent::PitchBend { channel, lsb, msb },
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            MidiEvent::NoteOff {
                channel,
                note,
                velocity,
            } => format!("[ch{channel}][NoteOff] note={note} velocity={velocity}"),
            MidiEvent::NoteOn {
                channel,
                note,
                velocity,
            } => format!("[ch{channel}][NoteOn] note={note} velocity={velocity}"),
            MidiEvent::PolyphonicPressure {
                channel,
                note,
                pressure,
            } => format!("[ch{channel}][Pressure] note={note} pressure={pressure}"),
            MidiEvent::Controller {
                channel,
                controller,
                value,
            } => format!("[ch{channel}][Controller] controller={controller} value={value}"),
            MidiEvent::ProgramChange { channel, program } => {
                format!("[ch{channel}][ProgramChange] program={program}")
            }

            MidiEvent::ChannelPressure { channel, pressure } => {
                format!("[ch{channel}][ChannelPressure] pressure={pressure}")
            }
            MidiEvent::PitchBend { channel, lsb, msb } => {
                format!("[ch{channel}][PitchBend] lsb={lsb} msb={msb}")
            }
        }
    }
}

/// A midi event
///
/// Normally, the majority of messages will be of this type. They are the key messages for
/// starting and stopping sound, along with changing pitch.
///
/// Note that for all values, the top bit is not used, so the numbers will be interpreted the same
/// for either u8 or i8. I use u8 here.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MidiEvent {
    NoteOff {
        channel: u8,
        note: Note,
        velocity: u8,
    },

    NoteOn {
        channel: u8,
        note: Note,
        velocity: u8,
    },
    /// Apply aftertouch pressure to the given note
    ///
    /// The second param is the amount of aftertouch
    PolyphonicPressure {
        channel: u8,
        note: Note,
        pressure: u8,
    },
    /// Set a controller to a value
    ///
    /// The first param is the controller to set, and the second param is the value to set it to
    Controller {
        channel: u8,
        controller: u8,
        value: u8,
    },
    /// Select the specified program
    ///
    /// The second param is the program to set.
    ProgramChange { channel: u8, program: u8 },
    /// Allows all notes to have a specific aftertouch used as default, similar to
    /// `PolyphonicPressure`
    ChannelPressure { channel: u8, pressure: u8 },
    /// Apply pitch bend to all notes
    ///
    /// First param is less significant byte, and second is most significant byte. The value of
    /// `0x00 0x40` means 'no bend', less means bend down and more means bend up.
    PitchBend { channel: u8, lsb: u8, msb: u8 },
}

pub fn parse_midi_event(i: &[u8]) -> IResult<&[u8], MidiEvent> {
    let (i, code_chan) = be_u8(i)?;

    let event_type = code_chan >> 4;
    let channel = code_chan & 0x0F;

    let result = match event_type {
        0x8 => {
            let (i, note_code) = utils::be_u7(i)?;
            let (i, velocity) = utils::be_u7(i)?;
            (
                i,
                MidiEvent::NoteOff {
                    channel,
                    note: note_code.into(),
                    velocity,
                },
            )
        }

        0x9 => {
            let (i, note_code) = utils::be_u7(i)?;
            let (i, velocity) = utils::be_u7(i)?;
            (
                i,
                MidiEvent::NoteOn {
                    channel,
                    note: note_code.into(),
                    velocity,
                },
            )
        }

        0xA => {
            let (i, note_code) = utils::be_u7(i)?;
            let (i, pressure) = utils::be_u7(i)?;
            (
                i,
                MidiEvent::PolyphonicPressure {
                    channel,
                    note: note_code.into(),
                    pressure,
                },
            )
        }

        0xB => {
            let (i, controller) = be_u8(i)?;
            let (i, value) = utils::be_u7(i)?;
            (
                i,
                MidiEvent::Controller {
                    channel,
                    controller,
                    value,
                },
            )
        }

        0xC => {
            let (i, program) = utils::be_u7(i)?;
            (i, MidiEvent::ProgramChange { channel, program })
        }

        0xD => {
            let (i, pressure) = utils::be_u7(i)?;
            (i, MidiEvent::ChannelPressure { channel, pressure })
        }

        0xE => {
            let (i, lsb) = utils::be_u7(i)?;
            let (i, msb) = utils::be_u7(i)?;
            (i, MidiEvent::PitchBend { channel, lsb, msb })
        }

        0xFA => {
            dbg!("--------------");
            dbg!(i);
            return Err(Err::Error(make_error(i, ErrorKind::Digit)));
        }

        _ => return Err(Err::Error(make_error(i, ErrorKind::Digit))),
    };

    Ok(result)
}
