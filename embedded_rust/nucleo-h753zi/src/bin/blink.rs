#![no_main]
#![no_std]

use nucleo_h753zi as _;
use stm32h7xx_hal::{pac, prelude::*};

#[cortex_m_rt::entry]
fn main() -> ! {
    
    let dp = pac::Peripherals::take().unwrap();
    
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    let rcc = dp.RCC.constrain();
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let mut led = gpioe.pe1.into_push_pull_output();

    let cp = cortex_m::Peripherals::take().unwrap();
    let mut delay = cp.SYST.delay(ccdr.clocks);

    loop
    {
        led.set_high();
        delay.delay_ms(500_u16);

        led.set_low();
        delay.delay_ms(500_u16);
    }
}
