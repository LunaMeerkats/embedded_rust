# Embedded Rust on STM32H753: build, flash, debug, and print through GDB

This guide records the working setup used on 29 August 2026. It explains how the pieces fit together and gives a repeatable workflow for debugging a Rust program on the NUCLEO-H753ZI from WSL.

## Working setup

| Component | Value |
| --- | --- |
| Development board | STMicroelectronics NUCLEO-H753ZI |
| Microcontroller | STM32H753ZIT6 |
| CPU | Arm Cortex-M7F |
| Rust target | `thumbv7em-none-eabihf` |
| Debug probe | Onboard STLINK-V3 using SWD |
| Linux environment | Ubuntu under WSL 2 |
| Debug server | OpenOCD 0.12.0 |
| Debugger | `gdb-multiarch` |
| GDB server address | `localhost:3333` |
| Application directory | `/home/dan/projects/learning_rust/embedded_rust/app` |
| Debug ELF | `target/thumbv7em-none-eabihf/debug/hello` |

## Mental model

There are several separate tools involved:

1. Cargo and `rustc` compile the Rust source into an ELF file for the Cortex-M7F.
2. GDB reads that ELF file to obtain machine code, source lines, symbols, and debugging information.
3. OpenOCD controls the STLINK-V3 probe and exposes a GDB server on TCP port 3333.
4. The STLINK-V3 communicates with the STM32H753 over SWD.
5. Semihosting lets firmware request services such as printing from the attached debugger.

The control path is:

```text
GDB -> OpenOCD on port 3333 -> STLINK-V3 -> SWD -> STM32H753
```

For semihosted output forwarded into GDB, the return path is:

```text
Rust semihosting call -> CPU debug trap -> OpenOCD -> GDB File-I/O -> GDB console
```

## 1. Give WSL access to the STLINK-V3

WSL 2 does not receive USB devices automatically. Run the following in Windows PowerShell, not inside WSL.

List the devices:

```powershell
usbipd list
```

Find the ST-Link entry and note its bus ID. The observed bus ID was `2-4`, but it can change, so always check rather than assuming.

Binding normally needs an elevated PowerShell window and is generally a one-time operation for that device:

```powershell
usbipd bind --busid 2-4
```

Attach the bound device to WSL:

```powershell
usbipd attach --wsl --busid 2-4
```

While attached to WSL, the probe is not simultaneously available to Windows applications.

In WSL, verify that Linux can see it:

```bash
lsusb
```

The working device appeared as:

```text
0483:374e STMicroelectronics STLINK-V3
```

## 2. Prepare the Rust target and tools

Install the Rust compilation target:

```bash
rustup target add thumbv7em-none-eabihf
```

This target means:

- `thumb`: the Arm Thumb instruction set used by Cortex-M processors;
- `v7em`: Armv7E-M, which covers Cortex-M4 and Cortex-M7;
- `eabihf`: the embedded ABI with hardware floating-point calling conventions.

The working flow also needs OpenOCD and a GDB capable of understanding Arm targets:

```bash
sudo apt update
sudo apt install openocd gdb-multiarch usbutils
```

Useful Rust inspection tools can be installed with:

```bash
cargo install cargo-binutils
rustup component add llvm-tools
```

## 3. Understand the project configuration

The project selects the correct Rust target in `.cargo/config.toml`:

```toml
[build]
target = "thumbv7em-none-eabihf"
```

The OpenOCD configuration in `openocd.cfg` is:

```tcl
# NUCLEO-H753ZI / STM32H753ZIT6

source [find interface/stlink.cfg]
transport select hla_swd

source [find target/stm32h7x_dual_bank.cfg]

reset_config srst_only
```

This selects the ST-Link interface, SWD transport, and STM32H7 dual-bank target support.

## 4. Enable semihosted standard output

The `semihosting` crate keeps standard I/O support behind an optional feature. In `Cargo.toml`, use:

```toml
semihosting = { version = "0.1.20", features = ["stdio"] }
```

The `stdio` feature enables macros such as `semihosting::print!` and `semihosting::println!` in this `no_std` application.

The hello-world program in `src/bin/hello.rs` is:

```rust
#![no_main]
#![no_std]

use app as _; // global logger + panic behavior + memory layout

#[cortex_m_rt::entry]
fn main() -> ! {
    semihosting::println!("Hello, world!");

    app::exit()
}
```

Important details:

- `#![no_std]` avoids the desktop Rust standard library, which is unavailable on the bare-metal MCU.
- `#![no_main]` avoids the normal operating-system program entry point.
- `#[cortex_m_rt::entry]` marks the embedded entry function used after reset initialization.
- `semihosting::println!` sends a standard-output request through the debugger.
- `app::exit()` also uses semihosting to report successful termination.

### Why `defmt::println!` did not appear in GDB

`defmt::println!` and `semihosting::println!` use different output systems.

- `defmt::println!` writes compact encoded records through RTT. A defmt-aware tool such as probe-rs or cargo-embed must read and decode them.
- `semihosting::println!` asks the debugger host to perform an output operation. OpenOCD can forward that operation into GDB.

GDB can stop on a `defmt::println!` call, but it does not decode defmt's RTT data by itself.

## 5. Build the debug ELF

From the application directory:

```bash
cd /home/dan/projects/learning_rust/embedded_rust/app
cargo build --bin hello
```

The resulting file is:

```text
target/thumbv7em-none-eabihf/debug/hello
```

This is an ELF file, not just a raw flash image. It contains the program plus source-level debugging information that GDB needs. The working ELF was an Arm executable with `.debug_info` and was not stripped.

To inspect the release binary's sections and sizes:

```bash
cargo size --bin hello --release -- -A
```

Common ELF sections include:

- `.text`: executable instructions;
- `.rodata`: read-only constants such as strings;
- `.data`: initialized static variables;
- `.bss`: zero-initialized static variables;
- `.vector_table`: reset and interrupt vectors;
- `.debug_*`: debugger metadata, which is not flashed as runtime program data.

## 6. Start OpenOCD

Open a WSL terminal and run:

```bash
cd /home/dan/projects/learning_rust/embedded_rust/app
openocd -f openocd.cfg
```

Leave this terminal running. OpenOCD normally opens:

- port 3333 for GDB;
- port 4444 for its Telnet command interface;
- port 6666 for its Tcl interface.

Only one OpenOCD process should control the probe at a time. An `Address already in use` message usually means another OpenOCD process is already running.

## 7. Connect GDB, flash, and stop at the print line

Open a second WSL terminal:

```bash
cd /home/dan/projects/learning_rust/embedded_rust/app
gdb-multiarch -q target/thumbv7em-none-eabihf/debug/hello
```

At the `(gdb)` prompt, enter:

```gdb
target extended-remote localhost:3333
monitor reset halt
load
monitor arm semihosting enable
monitor arm semihosting_fileio enable
break src/bin/hello.rs:8
monitor reset halt
continue
```

Command explanations:

| Command | Meaning |
| --- | --- |
| `target extended-remote localhost:3333` | Connect GDB to OpenOCD's GDB server. |
| `monitor reset halt` | Ask OpenOCD to reset the MCU and leave its core stopped. |
| `load` | Program the ELF's loadable sections into the target's flash and RAM. This changes the board's firmware. |
| `monitor arm semihosting enable` | Tell OpenOCD to catch Arm semihosting requests. |
| `monitor arm semihosting_fileio enable` | Forward semihosting I/O through GDB's File-I/O protocol so console text appears in GDB. |
| `break src/bin/hello.rs:8` | Set a source breakpoint immediately before the print call. |
| `continue` | Resume the MCU until it reaches a breakpoint, fault, exit, or manual interrupt. |

A source-line breakpoint is preferable here because the public `main` symbol can refer to a `cortex-m-rt` wrapper compiled without source debugging information. Line 8 resolves to the actual Rust entry function.

## 8. Execute the print statement

When GDB stops at line 8, inspect the area around the current line:

```gdb
list
```

Execute the print statement and stop at the next source line:

```gdb
next
```

The GDB console should display:

```text
Hello, world!
```

The program should then be positioned at `app::exit()`. Finish it with:

```gdb
continue
```

Semihosting File-I/O is synchronous: output is serviced while GDB is executing a command such as `continue`, `next`, or `step`, not while the MCU is halted at an idle GDB prompt.

## Useful GDB commands

| Command | Purpose |
| --- | --- |
| `list` | Show source code around the current execution point. |
| `break file.rs:line` | Stop at a source line. |
| `tbreak file.rs:line` | Set a breakpoint that removes itself after the first hit. |
| `info breakpoints` | List breakpoints and their state. |
| `next` | Execute the current source line, stepping over function calls. |
| `step` | Execute the current source line, stepping into function calls where debug data permits. |
| `stepi` | Execute exactly one machine instruction. |
| `continue` | Resume execution. |
| `interrupt` or `Ctrl+C` | Halt a running target and return control to GDB. |
| `info registers` | Display CPU registers. |
| `x/8i $pc` | Disassemble eight instructions starting at the program counter. |
| `backtrace` | Display the call stack when usable unwind information is available. |
| `layout src` | Open GDB's terminal source-code layout. |
| `layout split` | Show source and assembly together. |
| `quit` | Disconnect and leave GDB. |

Do not use GDB's ordinary desktop-style `run` command for this flow. Reset the embedded target with an OpenOCD `monitor` command and then use `continue`.

## Troubleshooting

### WSL cannot see the ST-Link

Run `usbipd list` in Windows PowerShell and attach the current bus ID again. Then verify with `lsusb` in WSL. Keep at least one WSL session open while attaching.

### GDB reports connection refused on port 3333

OpenOCD is not listening, exited with an error, or was started with a different configuration. Check the OpenOCD terminal before starting GDB.

### OpenOCD cannot claim the USB device

Make sure the ST-Link is attached to WSL and not already owned by another OpenOCD, probe-rs, or Windows debugging process.

### The old program still seems to run

Rebuild, start GDB with the matching ELF, and issue `load` again:

```bash
cargo build --bin hello
gdb-multiarch -q target/thumbv7em-none-eabihf/debug/hello
```

GDB needs the ELF that exactly matches the firmware loaded on the board; otherwise source lines, variables, and addresses can be misleading.

### The message appears in the OpenOCD terminal instead of GDB

Check the two settings from GDB:

```gdb
monitor arm semihosting
monitor arm semihosting_fileio
```

Both should report enabled. Without File-I/O forwarding, OpenOCD handles the output itself, so the text may appear in the OpenOCD terminal.

### GDB stops in runtime assembly or cannot show source for `main`

Use a source breakpoint:

```gdb
break src/bin/hello.rs:8
```

The exported `main` symbol may be the runtime wrapper rather than the Rust function body.

### The program faults when no debugger is attached

Semihosting is a debugging technique, not a standalone production output channel. Its special instruction must be handled by a connected debugger. For normal standalone output, use a UART, USB serial, or another hardware-backed interface.

### `cargo run` cannot find or configure probe-rs

The current `.cargo/config.toml` still contains a template runner with the literal placeholder `$CHIP`. Cargo does not shell-expand that array entry. Replace it with a probe-rs chip identifier only after installing probe-rs and confirming the exact identifier with `probe-rs chip list`. The OpenOCD/GDB flow described above does not depend on that runner.

## Short repeatable session

After the one-time setup and source changes, the normal workflow is:

1. Attach the STLINK-V3 to WSL with `usbipd` if necessary.
2. Build in the app directory:

   ```bash
   cargo build --bin hello
   ```

3. Start OpenOCD in terminal 1:

   ```bash
   openocd -f openocd.cfg
   ```

4. Start GDB in terminal 2:

   ```bash
   gdb-multiarch -q target/thumbv7em-none-eabihf/debug/hello
   ```

5. Connect, flash, enable semihosting, and debug:

   ```gdb
   target extended-remote localhost:3333
   monitor reset halt
   load
   monitor arm semihosting enable
   monitor arm semihosting_fileio enable
   break src/bin/hello.rs:8
   monitor reset halt
   continue
   next
   ```

The important distinction to remember is that the ELF gives GDB an understanding of the program, OpenOCD gives GDB control of the hardware, and semihosting gives the firmware a debugger-assisted output channel.
