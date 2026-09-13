//| Crate                | Level           | Brief description                                                                                                                                              |
//| -------------------- | --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
//| `cortex-m`           | CPU             | Low-level access to the ARM Cortex-M processor core itself, including things like the NVIC, SysTick, SCB, interrupts, critical sections, and CPU instructions. |
//| `cortex-m-rt`        | Runtime         | Provides the Cortex-M startup/runtime support: reset handling, vector table integration, memory initialisation, exception handling, and `#[entry]`.            |
//| `stm32h7xx-hal::pac` | MCU registers   | Peripheral Access Crate for the STM32H7. Gives strongly typed, low-level access to the MCU's hardware registers and peripherals.                               |
//| `stm32h7xx-hal`      | MCU abstraction | Hardware Abstraction Layer for STM32H7 devices. Wraps the PAC in safer, higher-level APIs for GPIO, clocks, timers, SPI, UART, ADC, etc.                       |
//| `nucleo_h752zi`      | Board           | Board Support Package for the Nucleo-H753ZI. Adds board-specific definitions and configuration on top of the STM32H753 HAL.                                    |

#![no_main] // This tells rust to not generate a standard entry point or look for the main function.
#![no_std] // This tells rust to not include the standard library.

use nucleo_h753zi as _; 
// This does a number of things like:
//  link the runtime, vector table, and memory layout definitions.

use stm32h7xx_hal::{pac, prelude::*};
// From the hardware abstraction layer, we are grabbing the Peripoheral Access crate
// Using the prelude it brings into scope the most commonly used traits and types in the HAL.

#[cortex_m_rt::entry] // This tells the cortex-m-rt runtime where our code begins.

fn main() -> ! {
    
    let dp = pac::Peripherals::take().unwrap(); //dp stands for device peripherals.
    
    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.freeze();

    let rcc = dp.RCC.constrain(); //Reset and Clock Control
    let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG); //Core Clock Distribution and Reset

    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE); //Splits the peripheral into individual HAL
                                                       //pin objects.
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let _gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
                                                       
    let mut led = gpioe.pe1.into_push_pull_output();
    let button = gpioc.pc13.into_pull_down_input();
    
    let cp = cortex_m::Peripherals::take().unwrap(); //Core perihperals.
    let mut delay = cp.SYST.delay(ccdr.clocks);

    let _led_status:bool = false;

    loop
    {
        delay.delay_ms(500_u16);
        if button.is_high(){
            led.set_high();
        }        
        else{
            led.set_low();
        }
    }
}
