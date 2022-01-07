pub use fugit::{ExtU32, TimerInstantU32};

pub fn now() -> TimerInstantU32<1_000_000> {
    unsafe { external_now() }
}

extern "Rust" {
    fn external_now() -> TimerInstantU32<1_000_000>;
}
