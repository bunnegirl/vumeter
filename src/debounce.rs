use fugit::{ExtU32, Instant};
use crate::app::monotonics;

#[derive(Debug)]
pub struct Debounce<const DELAY_MS: u32>(pub Option<Instant<u32, 1_u32, 8000000_u32>>);

pub trait Debouncer<const DELAY_MS: u32> {
    fn new() -> Debounce<DELAY_MS>;
    fn update(&mut self);
    fn reset(&mut self);
    fn is_bouncing(&self) -> bool;
}

impl<const DELAY_MS: u32> Debouncer<DELAY_MS> for Debounce<DELAY_MS> {
    fn new() -> Debounce<DELAY_MS> {
        Debounce(Some(monotonics::now() + DELAY_MS.millis()))
    }

    fn update(&mut self) {
        if let Debounce(Some(debounce)) = self {
            self.0 = Some(*debounce + DELAY_MS.millis());
        } else {
            self.reset();
        }
    }

    fn reset(&mut self) {
        self.0 = Some(monotonics::now() + DELAY_MS.millis());
    }

    fn is_bouncing(&self) -> bool {
        if let Debounce(Some(debounce)) = self {
            return monotonics::now() < *debounce;
        }

        false
    }
}