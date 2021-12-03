pub use heapless::HistoryBuffer;

pub type Buffer = HistoryBuffer<f32, 500>;

pub trait BufferAvg {
    fn avg(&mut self) -> f32;
}

impl BufferAvg for Buffer {
    fn avg(&mut self) -> f32 {
        self.as_slice()
            .iter()
            .fold(0.0, |avg, val| {
                match *val {
                    x if x > 99.0 => avg,
                    x if x > 66.0 => (avg + x) / 2.0,
                    x if x > 55.0 => (avg + x) / 2.0,
                    x if x > 44.0 => (avg + x) / 2.0,
                    x if x > 33.0 => (avg + x) / 2.0,
                    x if x > 22.0 => (avg + x) / 2.0,
                    x if x > 11.0 => (avg + x) / 2.0,
                    _ => avg,
                }
            })
    }
}