mod app;
mod iter_utils;
mod midi_event;
mod midi_mapper;
mod parser;
mod pipeline;
mod scheduler;
mod tempo;
mod transforms;

use crate::app::AppConfig;
use app::App;
use clap::{Parser, Subcommand};
use midi_mapper::MidiMapper;
use parser::test_parse;
use schemars::schema_for;

/// TODO
#[derive(Debug, Parser)]
#[command(name = "TODO")]
#[command(about = "TODO", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Start {
        /// Config file path
        #[arg(short, long)]
        config_file: String,
    },
    Devices,
    Schema,
}

// How many threads should I use here...?
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Start { config_file } => {
            let config = test_parse(config_file).unwrap();
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

        Commands::Devices {} => {
            MidiMapper::print_ports();
        }

        Commands::Schema {} => {
            let schema = schema_for!(AppConfig);
            println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        }
    }
}
