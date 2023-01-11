use crate::Wrapper;

pub trait Transform {
    // This triggers on what we subscribe as points of interest, e.g. an arpeggio?
    fn on_tick(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        None
    }

    fn on_message(&mut self, v: Wrapper<u64>) -> Option<Wrapper<u64>> {
        Some(v)
    }

    fn process_message(&mut self, message: Wrapper<u64>) -> Option<Wrapper<u64>> {
        match message {
            Wrapper::Tempo => self.on_tick(message),
            Wrapper::Value(_) => self.on_message(message),
        }
    }
}
