# midori

Name extremely temporary. But it's something on Mi-Do... something?

## What

Do you think music was missing a certain something, maybe bugs? well boy do I have the thing for you.
A MIDI router/mapper/filter/transform etc for all your MIDI needs.

For instance, with the following YAML configuration file will:

- Take all note events (note on and off)
- Arpeggiate them on 1/8th notes
- Take each of those note on events and distribute them over channels, 2,3,4
- Output the notes on Electron Model Cycles

```yaml
input_devices:
  op1: "OP-1"

output_devices:
  emc: "Elektron Model:Cycles"

pipelines:
  - transforms:
      # Only take note events
      - type: Filter
        event_types:
          - NoteOff
          - NoteOn

      - type: Arpeggio
        note_duration: 200
        subdivision: 0.125 # 1/8th notes
        direction: Forward

      - type: Distribute
        # Each note will be distributed in order among these channels
        between: [2, 3, 4]

      - type: Output
        output_device: emc
```

## Why

While playing with some of my devices I found it lacking or too difficult on how I could route midi signals from one place to another. This only solves the lacking part, not too sure about the difficulty.

Still unsure whether this is useful, but at least it was fun to code.

## Caveats

Besides general wonkiness, this utility currently has no way of taking nor emitting clock events, and as such the tempo is currently fixed.

## Installation

No installation methods yet. For now, I'd suggest cloning, then doing `cargo run`

## Commands

### `start`

Starts the mapper with the given configuration

```sh
cargo run -- start --config-file=./path/to/your/file.yaml
```

### `devices`

Prints a list of the available input/output devices.
You use those names in the configuration file.

## Docs

Not yet that's for sure. But here's a list of the available transforms. Some more complete than others.

### `pipelines`

These are transformation pipelines. Events come in, are processed and then output.
Pipelines contain `transforms`, which will do things with the midi events coming in. Current list of `transforms`:

- `Arpeggio` can arpeggiate the current chord
- `Distribute` will distribute notes between multiple channels (can be useful to make monophonic synths with multiple channels into a polyphonic-ish)
- `Filter` filters by `event_types`, `channel`, or `devices`.
- `Inspect` prints out any events coming into this transform. Useful to debug.
- `Map` maps an incoming event to a different `channel` or `cc`.
- `Mirror` will duplicate incoming events among the given `channels`
- `Output` outputs all events to a specific output device. This should be the last transform of every pipeline.
- `Wasm` allows you to use a wasm module as a transform. Look into `examples/wasm` for an example with AssemblyScript
- `Mirror` will mirror all incoming events to all the designated `channels`
