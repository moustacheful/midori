# WASM example

This allows you to write your own transforms, by using WASM. This means you can use any language that supports exporting to a WASM module and use it to transform the MIDI events coming into it.

This example uses [AssemblyScript](https://www.assemblyscript.org) to produce a transform.

To run this:

1. Install node
2. Install the npm dependencies in this directory by doing: `npm i`
3. Build the WASM module by doing `npm run asbuild:release`
4. Open distribute `wasm.yml` and edit the input/outputs to correspond to your devices.
5. Execute `cargo run -- start --config-file=./wasm.yml`
6. Hope it doesn't crash.
