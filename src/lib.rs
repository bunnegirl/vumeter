#![no_main]
#![no_std]
#![feature(concat_idents)]
#![feature(trait_alias)]

pub mod bus;
pub mod debounce;
pub mod meter;
pub mod timer;
pub mod state;

use core::panic::PanicInfo;
use rtt_target::*;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);

    loop {
        cortex_m::asm::nop();
    }
}