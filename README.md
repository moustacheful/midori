# midori

Name extremely temporary.

## What

Do you think music was missing a certain something, maybe bugs? well boy do I have the thing for you.
A MIDI router/mapper/filter/transform etc for all your MIDI needs.

For instance:

```yaml
input_devices:
  orba: "Artiphon Orba"

output_devices:
  emc: "Elektron Model:Cycles"

pipelines:
  - name: "Tilt to pitch ch2"
    transforms:
      # Take ONLY controller events from channel 0
      - !Filter
        channels: [0]
        event_types: [Controller]
      - !Map
        channels:
          - [0, 2] # Map from channel 0 to channel 2
        cc:
          - [1, 13] # Map cc 1 to 13
      - !Output emc # Send them to Model:Cycles

  # The following group of pipelines will send events from channel 1 to channels 3 and 4 at the same time
  - name: "Route events to Model:Cycles ch2"
    transforms:
      - !Map
        channels:
          - [1, 2]
      - !Output emc

  - name: "Route events to Model:Cycles ch3"
    transforms:
      - !Map
        channels:
          - [1, 3]
      - !Output emc

  - name: "Arpeggio ch2 only"
    transforms:
      - !Filter
        channels: [2]
      - !Arpeggio
        subdivision: 0.25 # 1/4ths
        direction: PingPong
        note_duration: 100
      - !Output emc
```

## Why

While playing with some of my devices I found it lacking or too difficult on how I could route midi signals from one place to another. This only solves the lacking part, not too sure about the difficulty.

Still unsure whether this is useful, but at least it was fun to code.

## Installation

No installation methods yet. For now, I'd suggest cloning.

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

- `Arpeggio` can arpeggiate the current chord
- `Distribute` will distribute notes between multiple channels (can be useful to make monophonic synths with multiple channels into a polyphonic-ish)
- `Filter` filters by `event_types` or `channel`
- `Inspect` prints out any events coming into this transform. Useful to debug.
- `Map` maps an incoming event to a different `channel` or `cc`.
- `Output` outputs all events to a specific output device. This should be the last transform of every pipeline.
- `Wasm` allows you to use a wasm module as a transform. Look into `examples/wasm` for an example with AssemblyScript
