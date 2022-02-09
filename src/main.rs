#![no_main]
#![no_std]

pub mod hardware;
pub mod runtime;

use core::panic::PanicInfo;
use cortex_m::asm::nop;
use rtt_target::*;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rprintln!("{}", info);

    loop {
        nop();
    }
}
