mod app;
mod iter_utils;
mod midi_event;
mod midi_mapper;
mod parser;
mod pipeline;
mod scheduler;
mod tempo;
mod transforms;

use app::App;
use clap::Parser;
use midi_mapper::MidiMapper;
use parser::test_parse;

/// TODO
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Config file path
    #[arg(short, long)]
    config_file: String,
}

// How many threads should I use here...?
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let args = Args::parse();
    let config = test_parse(args.config_file).unwrap();
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
