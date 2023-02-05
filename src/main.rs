mod app;
mod midi_event;
mod midi_mapper;
mod parser;
mod pipeline;
mod tempo;
mod transforms;

use app::App;
use midi_mapper::MidiMapper;
use parser::test_parse;

// How many threads should I use here...?
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let config = test_parse("./test.yml".to_string()).unwrap();
    let mut midi_mapper = MidiMapper::new();

    config.input_devices.iter().for_each(|(alias, full_name)| {
        midi_mapper.add_input(full_name.clone(), alias.clone());
    });

    config.output_devices.iter().for_each(|(alias, full_name)| {
        midi_mapper.add_output(full_name.clone(), alias.clone());
    });

    let app = App::from_config(config);

    midi_mapper.start(app);
}
