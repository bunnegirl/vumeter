#![no_main]
#![no_std]
#![feature(concat_idents)]
#![feature(trait_alias)]

mod bus;
mod debounce;
mod mcu;
mod meter;
mod mono_timer;
mod state;

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
