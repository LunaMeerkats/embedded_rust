# Blinking an LED with embedded Rust on the NUCLEO-H753ZI

This note records what I learned on 30 August 2026 while creating and debugging my first LED-blinking program from scratch. The final program was confirmed working on the physical NUCLEO-H753ZI board.

## Result

The program repeatedly switches the yellow user LED on for 500 milliseconds and off for 500 milliseconds.

| Item | Value |
| --- | --- |
| Board | STMicroelectronics NUCLEO-H753ZI |
| Microcontroller | STM32H753ZIT6 |
| Processor | Arm Cortex-M7F |
| Rust target | `thumbv7em-none-eabihf` |
| LED | LD2, yellow |
| GPIO | PE1 |
| Electrical behaviour | Active-high: HIGH is on, LOW is off |
| Program | `embedded_rust/nucleo-h753zi/src/bin/blink.rs` |

One complete on/off cycle takes one second, so the resulting square wave has a frequency of 1 Hz.

## Working program

```rust
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

    loop {
        led.set_high();
        delay.delay_ms(500_u16);

        led.set_low();
        delay.delay_ms(500_u16);
    }
}
```

## Bare-metal Rust

```rust
#![no_main]
#![no_std]
```

`no_std` means the program does not use Rust's normal standard library. A bare microcontroller has no operating system to provide facilities such as files, processes, or ordinary console output.

`no_main` means the program does not use the normal desktop-program startup sequence. The Cortex-M runtime supplies the reset and startup process instead.

```rust
#[cortex_m_rt::entry]
fn main() -> !
```

The `entry` attribute marks the function that runs after the runtime has initialized memory. The return type `!` means the function never returns. This is satisfied by the permanent `loop` at the end of the program.

## The project support crate

```rust
use nucleo_h753zi as _;
```

The Cargo package is named `nucleo-h753zi`, but hyphens become underscores when the crate is used from Rust code.

Importing it as `_` links the support library without introducing a name that the program needs to use directly. That library supplies shared embedded facilities including the RTT logger, panic behaviour, and HardFault handler.

## PAC, HAL, and the prelude

```rust
use stm32h7xx_hal::{pac, prelude::*};
```

The PAC, or Peripheral Access Crate, exposes the microcontroller's hardware peripherals and registers as Rust types.

The HAL, or Hardware Abstraction Layer, builds safer and more convenient operations on top of those registers. The prelude imports extension traits that make methods such as `constrain`, `split`, `MHz`, and `delay` available.

The path from the program to the LED is:

```text
Rust program -> STM32H7 HAL -> peripheral registers -> GPIO PE1 -> yellow LD2
```

## Exclusive peripheral ownership

```rust
let dp = pac::Peripherals::take().unwrap();
let cp = cortex_m::Peripherals::take().unwrap();
```

There are two different sets of peripherals:

- `dp` contains device-specific STM32 peripherals such as PWR, RCC, GPIOE, timers, and serial devices.
- `cp` contains peripherals belonging to the Cortex-M processor core, including SysTick (`SYST`).

`take()` provides exclusive ownership and normally succeeds only once. This prevents two independent parts of a safe Rust program from configuring the same peripheral at the same time. `unwrap()` extracts the peripherals and would panic if they had already been taken.

## Power and clock configuration

```rust
let pwr = dp.PWR.constrain();
let pwrcfg = pwr.freeze();
```

`constrain()` changes the raw PWR peripheral into the HAL's configuration builder. `freeze()` commits the configuration and returns a value representing the finished power setup.

```rust
let rcc = dp.RCC.constrain();
let ccdr = rcc.sys_ck(100.MHz()).freeze(pwrcfg, &dp.SYSCFG);
```

RCC is Reset and Clock Control. This requests a 100 MHz system clock and then freezes the clock configuration. The returned `ccdr` value contains the resulting clock information and controlled tokens used to enable peripherals safely.

The order matters: the clock setup needs the completed power configuration.

## Configuring PE1 as an output

```rust
let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
let mut led = gpioe.pe1.into_push_pull_output();
```

`split()` enables GPIOE and turns the port into individually owned pin values.

`into_push_pull_output()` consumes the original PE1 value and returns a new PE1 value whose Rust type records that it is now a push-pull output. This is an example of Rust's type-state pattern: invalid output operations are prevented until the pin has been converted to an output mode.

The binding must be `mut` because changing the voltage level changes the pin's hardware state.

For this board, PE1 controls yellow LD2. It is active-high:

- `led.set_high()` turns it on.
- `led.set_low()` turns it off.

## Producing a visible delay

```rust
let mut delay = cp.SYST.delay(ccdr.clocks);
```

SysTick is a timer built into the Cortex-M core. The HAL needs the frozen clock information so it can translate a duration such as 500 milliseconds into the correct number of processor ticks.

```rust
loop {
    led.set_high();
    delay.delay_ms(500_u16);

    led.set_low();
    delay.delay_ms(500_u16);
}
```

The loop keeps the firmware alive permanently. Without the delays, HIGH followed immediately by LOW would produce a pulse too short to see. Without the loop, the operations would happen only once.

The suffix in `500_u16` tells Rust that the number is an unsigned 16-bit integer, matching one of the delay API's supported duration types.

## What originally went wrong

The first confirmed compiler error was:

```text
error: cannot find macro `example_power` in this scope
```

The original program used:

```rust
let pwrcfg = example_power!(pwr).freeze();
```

`example_power!` is a private convenience macro used by examples inside the `stm32h7xx-hal` repository. It is not exported as part of the HAL's public API. Copying a line from an example did not also copy the example's private `utilities` module.

The standalone program instead uses the public API directly:

```rust
let pwrcfg = pwr.freeze();
```

Other problems in the first version were:

- The pin was called `LED` and was immutable. It needed to be a mutable binding such as `led`.
- A variable called `gpioa` actually contained GPIOE. Renaming it `gpioe` made the hardware mapping clear.
- HIGH was followed immediately by LOW, so there was no visible pause.
- The program called `exit()` instead of remaining in a loop.
- The core peripherals and SysTick delay had not yet been acquired.

This showed that there are different categories of failure:

1. A compiler error, such as a missing macro.
2. A configuration error, such as selecting the wrong chip feature.
3. A logical error, such as switching the LED too quickly to see.
4. A toolchain error, such as an unavailable linker, runner, or debug probe.

It is easier to solve these one at a time, beginning with the first exact compiler error.

## Important configuration follow-up

The current program has worked on the board, but `Cargo.toml` still enables the HAL feature `stm32h743v`.

The board contains an STM32H753 revision-V microcontroller, so the exact feature should be:

```toml
stm32h7xx-hal = { version = "0.16.0", features = ["stm32h753v", "rt"] }
```

The working result does not make the H743 selection accurate. Selecting the exact MCU keeps future peripheral definitions and conditional HAL behaviour aligned with the real hardware.

The `probe-rs` runner in `.cargo/config.toml` also still contains `$CHIP`. This is separate from the established OpenOCD and GDB workflow and must be completed before relying on `cargo run` through `probe-rs`.

## Building and debugging

Check the Rust code without linking:

```bash
cargo check --bin blink
```

Build the debug ELF:

```bash
cargo build --bin blink
```

The resulting ELF is:

```text
target/thumbv7em-none-eabihf/debug/blink
```

Start OpenOCD in one terminal:

```bash
openocd -f openocd.cfg
```

Start GDB in another terminal:

```bash
gdb-multiarch -q target/thumbv7em-none-eabihf/debug/blink
```

Connect, load, reset, and run:

```gdb
target extended-remote localhost:3333
monitor reset halt
load
monitor reset halt
continue
```

## Managing several terminals with tmux

`tmux` allows several terminal panes to run inside one terminal window.

Install and start a named session:

```bash
sudo apt install tmux
tmux new -s blink
```

Press `Ctrl+B`, release it, and then press:

| Key | Action |
| --- | --- |
| `%` | Split vertically |
| `"` | Split horizontally |
| Arrow key | Move between panes |
| `z` | Maximize or restore the current pane |
| `x` | Close the current pane |
| `d` | Detach from the session |

Reattach later with:

```bash
tmux attach -t blink
```

A useful embedded-development layout is:

1. Editor and Cargo commands.
2. OpenOCD.
3. GDB.

## Moving text files recursively

To collect text files into a `notes` directory while avoiding `.git` and the destination itself:

```bash
mkdir -p notes
find . -path './.git' -prune -o -path './notes' -prune -o -type f -iname '*.txt' -exec mv -n -t ./notes -- {} +
```

The command means:

- `find .` searches recursively from the current directory.
- `-path './.git' -prune` avoids descending into Git's internal data.
- `-o` means “or; otherwise evaluate the next branch.”
- `-path './notes' -prune` prevents files already moved into the destination from being found again.
- `-type f` selects regular files.
- `-iname '*.txt'` finds `.txt` files case-insensitively.
- `-exec ... {} +` passes the matching paths to `mv` in efficient batches.
- `mv -n` does not overwrite an existing same-named file.
- `-t ./notes` sets the destination directory.
- `--` marks the end of options.
- `{}` is replaced by the paths found.

This command flattens the directory structure. If two source files have the same name, `-n` leaves the later file in its original location rather than overwriting the first one.

Preview the affected files before moving them:

```bash
find . -path './.git' -prune -o -path './notes' -prune -o -type f -iname '*.txt' -print
```

## Main lessons

- Embedded firmware has no operating system, so it uses `no_std`, a runtime entry point, and a function that never returns.
- Rust models unique hardware peripherals through ownership.
- The PAC describes registers; the HAL provides safer operations; the prelude brings the required traits into scope.
- Power must be configured before clocks, and clocks before clock-dependent peripherals.
- GPIO type-state records whether a pin is an input, output, or alternate-function pin.
- Hardware state changes require mutable bindings.
- A visible blink needs both timing and repetition.
- Code from a dependency's examples can rely on private support modules that application code cannot import.
- The first compiler error is usually the best place to begin debugging.
- A program that works can still contain an inaccurate target configuration worth correcting.
- tmux keeps the editor, debug server, and debugger visible together.
- Recursive file operations should be previewed and should protect against overwrites and accidental traversal of their destination.

## References

- [STM32H7 Nucleo-144 boards (MB1364) user manual](https://www.st.com/resource/en/user_manual/um2407-stm32h7-nucleo144-board-stmicroelectronics.pdf)
- [stm32h7xx-hal project and supported device features](https://github.com/stm32-rs/stm32h7xx-hal)
- Existing local guide: `ai_summary/embedded-rust-gdb-semihosting.md`
