#![no_main]
#![no_std]

use nucleo_h753zi as _; 
use stm32h7xx_hal::{pac, prelude::*};

#[cortex_m_rt::entry]
fn main() -> ! {
    
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    
    let pwr = dp.PWR.constrain();
    let rcc = dp.RCC.constrain();

    let pwrcfg = pwr.freeze();
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
                                                       
    let mut led = gpioe.pe1.into_push_pull_output();
    let mut delay = cp.SYST.delay(ccdr.clocks);
    let button = gpioc.pc13.into_pull_down_input();

    let mut led_status:bool = false;

    loop
    {

        if button.is_high(){
            led_status = !led_status;
        }

        if led_status == true { 
            led.set_high();
            delay.delay_ms(500_u16);
        }

        if led_status == false {
            led.set_low();
            delay.delay_ms(500_u16);
        }

    }
}
