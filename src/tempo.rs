use futures;
use futures::Stream;
use futures::StreamExt;
use std::time::Duration;
use tokio_stream::wrappers::IntervalStream;

/**
 * This is some kind of internal clock?
 * e.g. when we have no device as source of it all.
 */
pub struct Tempo {
    pub(crate) main: IntervalStream,
    pub(crate) beat_interval: u64,
}

impl Tempo {
    pub(crate) fn get_interval_stream(interval_ms: u64) -> IntervalStream {
        let interval = tokio::time::interval(Duration::from_millis(interval_ms));

        tokio_stream::wrappers::IntervalStream::new(interval)
    }

    pub fn new(bpm: u64) -> Tempo {
        let beat_interval = ((bpm as f64 / 60.0) * 1000.0) as u64;
        let main = Self::get_interval_stream(beat_interval);

        Tempo {
            main,
            beat_interval,
        }
    }

    pub fn subdiv(&mut self, factor: u64) -> impl Stream<Item = u64> {
        // This seems wrong... but we want to remove the need for owning self at the end
        futures::executor::block_on(self.main.next());

        Self::get_interval_stream(self.beat_interval.clone() / factor).map(|_| 10 as u64)
    }
}
