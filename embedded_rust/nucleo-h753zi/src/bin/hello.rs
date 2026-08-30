#![no_main]
#![no_std]

use nucleo_h753zi as _; // global logger + panicking-behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    semihosting::println!("Hello, world!");

    nucleo_h753zi::exit()
}
