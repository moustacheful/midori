use midir::{
    ConnectError, MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection,
};

use crate::{
    midi_event::{MIDIEvent, MIDIRouterEvent, ToMidi},
    App,
};
use std::{collections::HashMap, error::Error};

pub struct MidiMapper {
    midi_sender: flume::Sender<MIDIRouterEvent>,
    ingress: flume::Receiver<MIDIRouterEvent>,

    input_connections: HashMap<String, MidiInputConnection<()>>,
    output_connections: HashMap<String, MidiOutputConnection>,
}

impl MidiMapper {
    pub fn new() -> MidiMapper {
        let (tx, rx) = flume::unbounded();

        MidiMapper {
            midi_sender: tx,
            ingress: rx,
            input_connections: HashMap::new(),
            output_connections: HashMap::new(),
        }
    }

    pub fn start(&mut self, mut app: App) {
        let (egress_sender, egress_receiver) = flume::unbounded::<MIDIRouterEvent>();

        app.set_egress(egress_sender);
        app.set_ingress(self.ingress.clone());

        tokio::spawn(async {
            app.run().await;
        });

        loop {
            let message = egress_receiver.recv().unwrap();

            match self.output_connections.get_mut(&message.device) {
                Some(output) => {
                    let midi_message = message.event.to_midi();
                    output.send(&midi_message).unwrap();
                }
                None => todo!(),
            };
            //println!("{message}");
        }
    }

    pub fn print_ports() {
        let midi_in = MidiInput::new("midir forwarding input").unwrap();
        let midi_out = MidiOutput::new("midir forwarding output").unwrap();

        fn print_ports_from<T: MidiIO>(m: T) {
            let midi_ports = m.ports();
            midi_ports.iter().for_each(|port| {
                println!("{}", m.port_name(port).unwrap());
            });
        }

        println!("Available input ports:");
        print_ports_from(midi_in);

        println!("");

        println!("Available output ports:");
        print_ports_from(midi_out);
    }

    fn select_port_by_name<T: MidiIO>(
        midi_io: &T,
        name: String,
    ) -> Result<T::Port, Box<dyn Error>> {
        let midi_ports = midi_io.ports();
        let port = midi_ports
            .iter()
            .find(|port| midi_io.port_name(port).unwrap().starts_with(&name))
            .expect(format!("Could not find port: '{name}'").as_str());

        Ok(port.clone())
    }

    pub fn add_input(&mut self, device_name: String, alias: String) {
        self.input_connections.insert(
            alias.clone(),
            self.connect_input(device_name, alias).unwrap(),
        );
    }

    fn connect_input(
        &self,
        name: String,
        alias: String,
    ) -> Result<MidiInputConnection<()>, ConnectError<MidiInput>> {
        let midi_in = MidiInput::new("midir forwarding input").unwrap();

        let port = Self::select_port_by_name(&midi_in, name).unwrap();
        let local_tx = self.midi_sender.clone();

        midi_in.connect(
            &port,
            "midir forward",
            move |_stamp, message, _| {
                let midi_event = MIDIEvent::try_from(message).expect("Could not parse midi event!");

                local_tx
                    .send(MIDIRouterEvent {
                        device: alias.clone(),
                        event: midi_event,
                    })
                    .unwrap_or_else(|_| println!("Error sending message to centralized bus"))
            },
            (),
        )
    }

    pub fn add_output(&mut self, device_name: String, alias: String) {
        self.output_connections
            .insert(alias, self.connect_output(device_name).unwrap());
    }

    fn connect_output(
        &self,
        name: String,
    ) -> Result<MidiOutputConnection, ConnectError<MidiOutput>> {
        let midi_out = MidiOutput::new("midir forwarding output").unwrap();
        let port = Self::select_port_by_name(&midi_out, name).unwrap();

        midi_out.connect(&port, "midir forward")
    }
}
