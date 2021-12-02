use heapless::Vec;

pub type Buffer = Vec<(f32, f32), 500>;

pub trait BufferAvg {
    fn avg(&mut self) -> (f32, f32);
}

impl BufferAvg for Buffer {
    fn avg(&mut self) -> (f32, f32) {
        self
            .iter()
            .fold((0.0, 0.0), |(avg_left, avg_right), (left, right)| {
                let left = match *left {
                    x if x > 99.0 => None,
                    x if x > 66.0 => Some((avg_left + x) / 2.0),
                    x if x > 55.0 => Some((avg_left + x) / 2.0),
                    x if x > 44.0 => Some((avg_left + x) / 2.0),
                    x if x > 33.0 => Some((avg_left + x) / 2.0),
                    x if x > 22.0 => Some((avg_left + x) / 2.0),
                    x if x > 11.0 => Some((avg_left + x) / 2.0),
                    _ => None,
                };
                let right = match *right {
                    x if x > 99.0 => None,
                    x if x > 66.0 => Some((avg_right + x) / 2.0),
                    x if x > 55.0 => Some((avg_right + x) / 2.0),
                    x if x > 44.0 => Some((avg_right + x) / 2.0),
                    x if x > 33.0 => Some((avg_right + x) / 2.0),
                    x if x > 22.0 => Some((avg_right + x) / 2.0),
                    x if x > 11.0 => Some((avg_right + x) / 2.0),
                    _ => None,
                };

                match (left, right) {
                    (Some(left), Some(right)) => {
                        (left, right)
                    }
                    (Some(left), None) => {
                        (left, avg_right)
                    }
                    (None, Some(right)) => {
                        (avg_left, right)
                    }
                    (None, None) => {
                        (avg_left, avg_right)
                    }
                }
            })
    }
}