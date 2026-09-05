# Next weekend: Rover Launch Control

**A two-day embedded Rust project for Saturday 5 and Sunday 6 September 2026**

- **Board:** NUCLEO-H753ZI
- **Microcontroller:** STM32H753ZIT6, Cortex-M7F
- **Core hardware:** blue B1 user button, yellow LD2, and optionally red LD3
- **Working directory:** <code>/home/dan/projects/learning_rust/embedded_rust/nucleo-h753zi</code>

## Mission

Build a small reaction game that feels like a rover launch console:

1. Press and release the blue button to arm the rover.
2. The board waits for an unpredictable-looking interval.
3. Yellow LD2 turns on: **GO!**
4. Press the button as quickly as possible.
5. An early press triggers a false-launch pattern.
6. A valid reaction time is printed and stored in an eight-score RAM history.
7. The firmware reports the best and average scores.

No breadboard or extra components are needed.

Do not try to absorb the whole guide on Friday night. During the weekend, read only the current milestone, reach its checkpoint, and then continue.

The project is deliberately small enough to finish, but it introduces the ideas that sit behind most embedded applications:

- GPIO input and output;
- pull resistors, electrical levels, edges, and switch bounce;
- polling and a state machine;
- blocking delays versus elapsed-time measurement;
- memory on the stack and in fixed arrays;
- flash, RAM, and memory-mapped peripheral registers;
- HAL and PAC responsibilities;
- debugger-aware logging and its timing cost;
- <code>Option</code>, <code>Result</code>, wrapping arithmetic, and useful invariants.

## The finish line

By Sunday afternoon, the project is complete when:

- five normal rounds can be played without resetting;
- an early press is detected as a false start;
- twenty ordinary presses create exactly twenty press events;
- reaction times are plausible and are not distorted by logging;
- eight scores are retained and the ninth replaces the oldest;
- best and average scores match a hand calculation;
- restarting the firmware constructs a fresh empty score history;
- GDB can show the score data changing in RAM;
- a separate PAC lab can set and reset LD2 through GPIOE BSRR;
- you can explain the path from a Rust method call to a hardware register.

Finishing those items matters more than reaching the stretch goals.

## A useful realisation: you have already written memory and registers

Your blink was not “before” memory or registers. It already used both, through safer layers:

| Existing action | What happened underneath |
| --- | --- |
| Creating and changing local variables | The compiler used CPU registers and RAM as appropriate |
| <code>led.set_high()</code> | The HAL performed a volatile write to GPIOE BSRR |
| <code>led.set_low()</code> | The HAL performed another BSRR write, using the reset half |
| Creating a SysTick delay | The HAL configured Cortex-M SysTick registers |
| Taking <code>pac::Peripherals</code> | Rust gave your program unique ownership of memory-mapped device peripherals |
| Taking <code>cortex_m::Peripherals</code> | Rust gave it unique ownership of Cortex-M core peripherals |

This weekend makes those effects visible. You will first use the HAL, then inspect ordinary RAM, and finally repeat one LED operation through the PAC.

## Hardware facts for this board

| Item | Default mapping | Electrical behaviour |
| --- | --- | --- |
| Yellow user LED LD2 | PE1 | Active-high: HIGH turns it on |
| Red user LED LD3 | PB14 | Active-high; optional stretch feedback |
| Blue user button B1 | PC13 | Released LOW, pressed HIGH on the stock board |
| Black button B2 | NRST | Hardware reset, not a user GPIO |

The MB1364 board normally connects B1 to PC13 through solder bridge SB51. Its schematic uses a pull-down and connects the pressed switch toward VDD, which makes the stock button active-high. A modified board can route B1 to PA0 through SB58, so the first exercise measures the real polarity instead of hiding an assumption.

Keep using LD2 as the core-project LED. LD1 has a solder-bridge-dependent PB0/PA5 mapping and adds no useful lesson yet.

## Weekend boundaries

The core project uses the crates already present. Do not add a random-number crate, async runtime, collection crate, or device driver.

The following are deliberately deferred:

- interrupts and shared interrupt state;
- RTIC or Embassy;
- internal flash erase/write;
- DMA and cache coherency;
- UART, I2C, and SPI;
- heap allocation;
- external sensors or displays.

There are stretch steps for several of these, but none is part of the finish line.

Also keep the known-working <code>blink.rs</code> unchanged. It is your recovery image and hardware sanity check.

## Crate map: what is already doing work

| Crate or module | Mental model | Use this weekend |
| --- | --- | --- |
| <code>core</code> | The no-OS foundation: arrays, enums, slices, arithmetic, <code>Option</code>, and <code>Result</code> | Game state and scoreboard |
| <code>cortex-m-rt</code> | Reset/startup, vector table, memory initialisation, exceptions, and <code>#[entry]</code> | Starts every binary |
| <code>cortex-m</code> | Safe access to Cortex-M core peripherals and instructions | SysTick delay and DWT cycle counter |
| <code>stm32h7xx-hal</code> | Type-safe STM32 power, clocks, GPIO, timers, and peripheral configuration | Most hardware work |
| <code>stm32h7xx_hal::pac</code> | Generated register blocks for the selected STM32H753 | The isolated register lab |
| <code>fugit</code> | Typed rates and durations | Already exposed through the HAL prelude, for example <code>100.MHz()</code> |
| <code>semihosting</code> | Debugger-mediated console I/O | State and score messages after timing is captured |
| <code>defmt</code> and <code>defmt-rtt</code> | Compact encoded target logging over RTT | Understand, but do not depend on it with plain GDB |
| <code>panic-probe</code> | Panic behaviour for probe-based debugging | Already installed by the support crate |
| <code>embedded-hal</code> | Portable capability traits used by HALs and drivers | Learn the purpose; do not add another version |

### One compatibility trap

The current <code>stm32h7xx-hal 0.16.0</code> uses the <code>embedded-hal 0.2.7</code> trait generation. The dependency tree also contains <code>embedded-hal 1.0</code> through <code>cortex-m</code>. They are different trait versions.

Do not add a direct <code>embedded-hal 1.x</code> dependency this weekend and assume that the current STM32 HAL implements those traits. When you later add a device driver, check its exact embedded-hal compatibility first.

### One logging trap

<code>defmt-rtt</code> sends encoded RTT frames. Ordinary GDB does not decode them. Your established OpenOCD/GDB workflow should use <code>semihosting::println!</code> for visible output.

Semihosting is intrusive: the CPU traps into the debugger. Never print inside the reaction-time interval. Capture the button timestamp first, then print the completed result.

Do not enable semihosting's <code>panic-handler</code> feature; this project already has panic behaviour through <code>panic-probe</code> and its support library.

## Architecture

~~~mermaid
flowchart LR
    B1["B1 / PC13"] --> IDR["GPIOC input register"]
    IDR --> HALIN["HAL input pin"]
    HALIN --> DEB["Debouncer + edge events"]
    DEB --> FSM["GameState enum"]
    DWT["Cortex-M DWT cycle counter"] --> FSM
    FSM --> SCORE["Scoreboard in RAM"]
    SCORE --> LOG["Semihosting after capture"]
    FSM --> HALOUT["HAL output pin"]
    HALOUT --> BSRR["GPIOE BSRR"]
    BSRR --> LD2["LD2 / PE1"]
~~~

The important layering is:

~~~text
Rover game
    ↓
HAL pin and clock APIs
    ↓
PAC register blocks
    ↓
volatile memory-mapped registers
    ↓
physical button and LED
~~~

## Weekend rhythm

| Time | Session | Physical checkpoint |
| --- | --- | --- |
| Saturday 09:30–10:15 | Protect and verify the baseline | Known blink still builds and runs |
| Saturday 10:15–11:15 | Button scout | LED mirrors the real button level |
| Saturday 11:30–12:30 | Debouncer | 20 presses become 20 events |
| Saturday 13:30–15:30 | Rover state machine | Game is playable, including false starts |
| Saturday 15:45–16:45 | RAM scoreboard | Results survive rounds, but not reset |
| Sunday 09:30–11:00 | DWT timing | Reaction score is measured in milliseconds |
| Sunday 11:15–12:15 | Memory microscope | GDB shows score data in RAM |
| Sunday 13:15–14:30 | PAC register lab | LD2 is controlled via BSRR |
| Sunday 14:45–16:00 | Acceptance run and notes | Finish-line checklist passes |

Take a short break whenever a checkpoint passes. Commit each known-good checkpoint before starting the next one.

---

## Saturday morning: protect the baseline

### 1. Attach the ST-Link to WSL

The ST-Link is not attached to WSL at the time this guide was written. In Windows PowerShell:

~~~powershell
usbipd list
~~~

Find the current ST-Link bus ID. Do not assume that the previous <code>2-4</code> value is unchanged.

If it has not been bound before, use an elevated PowerShell:

~~~powershell
usbipd bind --busid BUS-ID
~~~

Attach it:

~~~powershell
usbipd attach --wsl --busid BUS-ID
~~~

Then in WSL:

~~~bash
lsusb
~~~

Expected device identity:

~~~text
0483:374e STMicroelectronics STLINK-V3
~~~

If USB attachment is troublesome, use the detailed existing guide at <code>ai_summary/embedded-rust-gdb-semihosting.md</code>.

### 2. Create a learning branch

~~~bash
cd /home/dan/projects/learning_rust
git status --short --branch
git switch -c learning/rover-launch-control
git add ai_summary/next-weekend-rover-launch-control.md
git commit -m "Add Rover Launch Control weekend guide"
~~~

Before creating the branch, the only expected change is this untracked guide. If anything else appears, stop and understand it; do not discard an unexpected change. Commit the guide on the learning branch as the planning checkpoint.

### 3. Correct the MCU selection

The current program worked physically, but <code>Cargo.toml</code> still selects an H743 revision-V PAC. The part number <code>STM32H753ZIT6</code> alone does not distinguish silicon revision V from older revision Y, so confirm the board's recorded silicon revision before choosing the feature.

~~~toml
stm32h7xx-hal = { version = "0.16.0", features = ["stm32h743v", "rt"] }
~~~

to:

~~~toml
stm32h7xx-hal = { version = "0.16.0", features = ["stm32h753v", "rt"] }
~~~

Use <code>stm32h753v</code> for revision V and <code>stm32h753</code> for revision Y. The current board notes identify revision V; this check makes that evidence explicit. This is an accuracy gate, not a feature upgrade.

The probe-rs runner still contains literal <code>$CHIP</code>. Leave it alone and continue with the working OpenOCD/GDB workflow; <code>cargo run</code> through probe-rs is not part of this weekend.

### 4. Re-prove blink

~~~bash
cd /home/dan/projects/learning_rust/embedded_rust/nucleo-h753zi
cargo check --bin blink
cargo build --bin blink
~~~

Start OpenOCD in one terminal:

~~~bash
openocd -f openocd.cfg
~~~

Start GDB in another:

~~~bash
gdb-multiarch -q target/thumbv7em-none-eabihf/debug/blink
~~~

Load and run:

~~~gdb
target extended-remote localhost:3333
monitor reset halt
load
monitor arm semihosting enable
monitor arm semihosting_fileio enable
monitor reset halt
continue
~~~

The two semihosting commands are required before a binary using <code>semihosting::println!</code> can print through this GDB workflow.

**Checkpoint:** LD2 still blinks at the known rate. Commit the device-feature correction separately.

### 5. Start a separate binary

Create <code>src/bin/rover_launch.rs</code>. Reuse the known-good power, 100 MHz clock, PE1 output, and SysTick setup from blink, but leave <code>blink.rs</code> untouched.

Cargo will auto-discover the new file as a binary:

~~~bash
cargo check --bin rover_launch
~~~

**Checkpoint:** the new binary builds before game code is added.

---

## Saturday late morning: make the button observable

### Milestone A: button scout

Add GPIOC to the existing setup:

- split <code>dp.GPIOC</code> with the matching <code>ccdr.peripheral.GPIOC</code> token;
- convert PC13 into a pull-down input;
- read it with <code>is_high()</code>;
- keep the existing PE1 push-pull output;
- log only when the raw state changes.

The two key HAL shapes to discover through rust-analyzer or the crate docs are:

~~~rust
let gpioc = dp.GPIOC.split(/* matching RCC token */);
let button = gpioc.pc13.into_pull_down_input();
~~~

Do not paste a complete example. Use the compiler to finish the exact ownership flow.

For the first physical test, make LD2 mirror the button:

| Button | Expected raw state | LD2 |
| --- | --- | --- |
| Released | LOW / <code>false</code> | Off |
| Pressed | HIGH / <code>true</code> | On |

**Checkpoint:** ten press/release cycles have the expected polarity. Record the observed result in a comment before moving on.

If the state never changes:

1. confirm WSL can still see ST-Link;
2. confirm the correct binary was loaded;
3. confirm GPIOC was split using its own RCC token;
4. inspect board solder bridges SB51/SB58 before changing PC13 to PA0;
5. do not “fix” it by inverting the boolean until the electrical state is understood.

### Milestone B: turn levels into events

A held button is a level. A game needs events:

~~~text
released → pressed  = Pressed event
pressed → released  = Released event
no stable transition = no event
~~~

Mechanical contacts bounce. One physical press can look like several rapid HIGH/LOW transitions.

Build a small debouncer yourself:

1. sample every 5 ms;
2. remember the candidate raw state;
3. count consecutive samples equal to that candidate;
4. accept it only after three matching samples;
5. emit an event only when the accepted stable state changes.

Useful data model:

~~~rust
enum ButtonEvent {
    Pressed,
    Released,
}

struct Debouncer {
    stable: bool,
    candidate: bool,
    matching_samples: u8,
}
~~~

Let the update method return <code>Option&lt;ButtonEvent&gt;</code>:

- <code>None</code> means no new stable event;
- <code>Some(ButtonEvent::Pressed)</code> means one accepted press;
- <code>Some(ButtonEvent::Released)</code> means one accepted release.

This is a good use of <code>Option</code>: absence is represented by the type, not by a magic number.

**Checkpoint test:**

- press and release normally 20 times;
- holding the button must create one Pressed event, not a stream;
- the counter must finish at exactly 20;
- very fast taps may be filtered, and you should be able to explain why.

Commit the debouncer before adding the game.

---

## Saturday afternoon: make it a game

### Milestone C: model the rules as states

Use one main loop and one explicit state value. Avoid a permanent loop inside each phase.

A useful starting shape is:

~~~rust
enum GameState {
    Ready,
    Armed,
    Waiting {
        started_ms: u32,
        wait_ms: u32,
    },
    Go {
        started_ms: u32,
    },
    ShowingResult {
        score_ms: u32,
        started_ms: u32,
    },
    FalseStart {
        started_ms: u32,
    },
}
~~~

The fields make each state's required data explicit. For example, a Waiting state owns a wait duration, while a Go state owns the reaction start time.

Implement these transitions:

| Current state | Event or condition | Action | Next state |
| --- | --- | --- | --- |
| Ready | Pressed | Short “armed” flash | Armed |
| Armed | Released | Choose the wait and record start | Waiting |
| Waiting | Wait elapsed | Turn LD2 on and record GO time | Go |
| Waiting | Pressed before wait elapsed | Keep LD2 off | FalseStart |
| Go | Pressed | Capture time first, then turn LD2 off | ShowingResult |
| Go | Five seconds elapsed | Record timeout, no score | ShowingResult |
| ShowingResult | Feedback complete and button released | Print summary | Ready |
| FalseStart | Feedback complete and button released | Print false-start message | Ready |

Requiring release before Waiting prevents the arming press from also becoming a false start.

Define a deterministic boundary rule. A fair simple rule is:

1. check whether the wait has elapsed;
2. if it has, enter Go;
3. only treat a Pressed event as false while the state is still Waiting.

That makes a press on the exact GO boundary a zero-ish reaction rather than a false start.

### Milestone D: use a small software timebase first

Keep the 5 ms sampling loop from the debouncer:

1. sample the input;
2. update the debouncer;
3. match on the game state;
4. update LED output;
5. delay for 5 ms;
6. advance <code>now_ms</code with <code>wrapping_add(5)</code>.

Calculate elapsed time with:

~~~rust
let elapsed_ms = now_ms.wrapping_sub(started_ms);
~~~

Do not compare absolute timestamps with ordinary <code>&gt;</code> after a counter can wrap. Elapsed-time subtraction is the reusable embedded pattern.

For Saturday, use a deterministic wait table:

~~~rust
const WAIT_MS: [u32; 4] = [1_200, 1_850, 2_600, 3_400];
~~~

Choose the next element by round number. Call it “varied,” not random. True hardware randomness is unnecessary for the lesson.

### Give the LED a language

Use simple patterns that remain distinguishable:

| Meaning | LD2 pattern |
| --- | --- |
| Ready | One short heartbeat every two seconds |
| Armed | Two short flashes |
| Waiting | Off |
| GO | Solid on |
| False start | Six rapid flashes |
| New best | Three medium flashes |
| Normal result | One long flash |

Patterns may use short blocking delays only in feedback states. Waiting and Go must keep returning to the main loop so the button continues to be sampled.

### State-machine checkpoint

Play these paths deliberately:

1. normal reaction;
2. early press;
3. hold the button after arming;
4. do nothing after GO until timeout;
5. play another round without reset.

For each transition, be able to say:

- what triggered it;
- which timestamp belongs to the new state;
- whether the LED changed before or after the timestamp;
- why a press event can occur only once per physical press.

Commit the playable state machine before adding score history.

---

## Saturday late afternoon: write ordinary RAM

### Milestone E: fixed-capacity scoreboard

Use a fixed array. This is predictable, needs no allocator, and is a better first embedded data structure than <code>Vec</code>.

~~~rust
struct Scoreboard {
    scores_ms: [u32; 8],
    len: usize,
    next: usize,
}
~~~

Implement:

~~~text
new() -> Scoreboard
record(score_ms)
best() -> Option<u32>
average() -> Option<u32>
~~~

Required behaviour:

- while <code>len &lt; 8</code>, append into the unused part;
- after eight values, write at <code>next</code> and advance it as a ring;
- the ninth valid score replaces the oldest;
- false starts and timeouts are not recorded;
- <code>best()</code> and <code>average()</code> return <code>None</code> when empty;
- compute an average through a <code>u64</code> sum so addition cannot overflow as easily.

This structure is stored in RAM as part of <code>main</code>'s live state. Each assignment to the array changes ordinary memory. No “memory-writing crate” is required.

Optional useful error type:

~~~rust
enum RoundError {
    FalseStart,
    Timeout,
    ImpossibleScore,
}
~~~

Let score validation return <code>Result&lt;u32, RoundError&gt;</code>. Handle expected outcomes with <code>match</code>. Reserve panic for a broken program invariant, not a player pressing early.

### Scoreboard checkpoint

Feed known test values through real or temporarily scripted rounds:

~~~text
310, 275, 420, 290, 350, 260, 500, 330, 240
~~~

Check the best and average by hand after the eighth and ninth inserts. Verify that restarting the firmware produces an empty logical history.

A reset does not guarantee that every SRAM cell is physically erased. The local history is empty because <code>main</code> runs again and <code>Scoreboard::new()</code> constructs a fresh value. Separately, <code>cortex-m-rt</code> copies initialised <code>.data</code> values from flash and zeroes <code>.bss</code> during startup.

### Saturday stop condition

Stop for the day with:

- a working raw button reader;
- one-event-per-press debouncing;
- a playable state machine;
- false-start and timeout paths;
- a score history that resets with the board.

Do not begin register code late in the day. Commit, write down the first unresolved question, and leave the known-working game intact.

---

## Sunday morning: measure real elapsed cycles

Saturday's <code>now_ms += 5</code> is understandable but includes loop overhead and assumes every loop takes exactly 5 ms. Replace reaction measurement with the Cortex-M Data Watchpoint and Trace cycle counter, DWT CYCCNT.

This is a hardware counter in the processor core. It advances independently while your loop polls the button.

### Milestone F: enable and prove DWT

The Cortex-M crate requires trace to be enabled before its cycle counter:

~~~rust
let mut cp = cortex_m::Peripherals::take().unwrap();
cp.DCB.enable_trace();
cp.DWT.enable_cycle_counter();
~~~

Do this before moving <code>cp.SYST</code> into the HAL delay provider.

Read time with:

~~~rust
use cortex_m::peripheral::DWT;

let now_cycles = DWT::cycle_count();
~~~

Before using it for the game, prove that two reads separated by a small delay produce different values. If the counter stays at zero:

1. confirm <code>DCB::enable_trace()</code> ran first;
2. check <code>DWT::cycle_counter_enabled()</code>;
3. only then investigate whether <code>DWT::unlock()</code> is needed on the target.

Do not assume a debugger breakpoint is a valid timing experiment. A halted core changes what you are measuring.

### Milestone G: convert cycles correctly

Read the actual frozen CPU core-clock rate, because DWT CYCCNT counts core cycles:

~~~rust
let core_hz = ccdr.clocks.c_ck().raw();
~~~

Capture the GO timestamp immediately after making LD2 visible, with no print between them:

~~~text
set LD2 HIGH
capture DWT cycle count
enter Go state
~~~

On the accepted Pressed event:

~~~text
capture DWT cycle count
compute wrapping difference
turn LD2 LOW
validate and store score
only now print
~~~

The simplest implementation timestamps the accepted debounced event, so its score includes roughly 10–15 ms of debounce latency. Measure that bias and document it. For a fairer refinement, capture the cycle count on the first raw HIGH candidate, carry that timestamp through the debouncer, and use it only if the candidate becomes a valid stable Pressed event. Discard the timestamp if the candidate collapses as bounce.

Use:

~~~rust
let elapsed_cycles = pressed_at.wrapping_sub(go_at);
let elapsed_ms =
    (u64::from(elapsed_cycles) * 1_000) / u64::from(core_hz);
~~~

The <code>u64</code> intermediates prevent multiplication overflow.

Validate the result against the five-second game limit before converting it to the scoreboard's <code>u32</code> value.

At 100 MHz, a 32-bit cycle counter wraps in about 42.95 seconds. Your waiting and reaction windows are below five seconds, so <code>wrapping_sub</code> produces the correct elapsed value across one wrap.

### Milestone H: make the wait human-seeded

At the moment the arming button is released, capture the low bits of the free-running counter:

~~~rust
let seed = DWT::cycle_count();
let wait_ms = 1_000 + (seed % 3_001);
~~~

Human button timing makes the delay hard to predict. This is suitable for a game, but it is not cryptographic randomness and must not be described as such.

Convert wait milliseconds to cycles with a <code>u64</code> intermediate, or continue using the 5 ms software timebase for Waiting. Keep the reaction score itself on DWT.

### Timing checkpoint

Play ten rounds:

- ordinary human results should normally be in the hundreds of milliseconds;
- a deliberately slow result should be clearly larger;
- logging must occur only after <code>pressed_at</code> is captured;
- a breakpoint or semihosting call during Go should visibly ruin the experiment, which explains why it is forbidden in the final code.

Commit the DWT version only after it behaves more convincingly than the Saturday timebase.

---

## Sunday late morning: the memory microscope

### 1. Read the linker memory map

The current <code>memory.x</code> says:

~~~text
FLASH starts at 0x08000000 and has length 2M
RAM   starts at 0x20000000 and has length 128K
~~~

This linker file selects one 128 KiB DTCM region for the program's RAM. It does not describe every RAM bank physically present on the STM32H753.

Use:

~~~bash
cargo size --bin rover_launch -- -A
~~~

Interpret the important sections:

| Section | Lives in | Meaning |
| --- | --- | --- |
| <code>.vector_table</code> | Flash | Reset and exception vectors |
| <code>.text</code> | Flash | Machine instructions |
| <code>.rodata</code> | Flash | Read-only constants and strings |
| <code>.data</code> | RAM at runtime, initial image in flash | Initialised mutable statics |
| <code>.bss</code> | RAM | Zero-initialised mutable statics |
| <code>.uninit</code> | RAM | Intentionally uninitialised buffers, including RTT storage |
| <code>.debug_*</code> | ELF file on the PC | Debug information, not flashed as runtime code |

The large Total line includes debug information. Do not mistake it for flash consumption.

### 2. Inspect the scoreboard in GDB

Build and load the debug ELF. Put a source breakpoint immediately after a call to <code>scoreboard.record(...)</code>.

Useful GDB commands:

~~~gdb
info locals
p scoreboard
p scoreboard.scores_ms
p/x &scoreboard.scores_ms
x/8uw &scoreboard.scores_ms
continue
~~~

Play another valid round and examine the same address again.

The dev profile currently uses size optimisation. If GDB says a local was optimised out, make a one-off inspection build:

~~~bash
CARGO_PROFILE_DEV_OPT_LEVEL=0 cargo build --bin rover_launch
~~~

Reload that matching ELF before continuing. Never debug source against a different ELF.

### 3. Prove reset semantics

After at least two scores:

1. inspect the array;
2. issue <code>monitor reset halt</code>;
3. continue from reset;
4. break after initialisation;
5. inspect the scoreboard again.

Explain why flash still contains the program and why <code>Scoreboard::new()</code> recreates an empty history even though reset does not promise to erase all SRAM bits.

### Memory checkpoint

Without looking at the table, explain:

- why <code>const WAIT_MS</code> does not behave like the mutable scoreboard;
- why a local array can live in RAM without <code>static mut</code>;
- why a firmware restart constructs an empty scoreboard without requiring a physical SRAM erase;
- why GPIO registers are accessed through addresses but are not ordinary RAM;
- why persistent flash storage is a separate, more hazardous exercise.

---

## Sunday afternoon: PAC register x-ray

Create <code>src/bin/register_lab.rs</code>. Keep it separate from <code>rover_launch.rs</code>.

The purpose is not to replace the HAL. It is to see one layer beneath it and then return to the HAL for application code.

### Safety rules

- Use the HAL-re-exported <code>pac</code>; do not add a separate STM32 PAC dependency.
- Use generated field methods, not hard-coded peripheral addresses.
- Do not use <code>Peripherals::steal()</code>.
- Do not mix a HAL-owned PE1 pin and independent PAC writes to PE1 in this binary.
- Do not touch FLASH control, option bytes, watchdogs, or unrelated RCC fields.
- Keep the known-good blink and finished game unchanged.

### Register worksheet

Use the reset clock for this tiny lab and work through these generated fields:

| Peripheral register | Field | Required value | Purpose |
| --- | --- | --- | --- |
| RCC AHB4ENR | GPIOEEN | Enabled | Supply the GPIOE peripheral clock |
| GPIOE MODER | MODER1 | Output | Make PE1 a general-purpose output |
| GPIOE OTYPER | OT1 | Push-pull | Match the LED's HAL configuration |
| GPIOE BSRR | BS1 | Set bit | Drive PE1 HIGH and turn LD2 on |
| GPIOE BSRR | BR1 | Set bit | Drive PE1 LOW and turn LD2 off |

The intended PAC method shapes are:

~~~text
ahb4enr.modify(... gpioeen().enabled())
moder.modify(... moder1().output())
otyper.modify(... ot1().push_pull())
bsrr.write(... bs1().set())
bsrr.write(... br1().set())
~~~

After enabling GPIOE in AHB4ENR, read the enable register back before accessing GPIOE. The read both confirms the field and gives the peripheral clock enable time to take effect.

Use <code>cortex_m::asm::delay</code> only to make the two states visible. Exact wall-clock accuracy is not the lesson in this binary.

### Understand read, write, and modify

| PAC operation | Meaning | Appropriate use here |
| --- | --- | --- |
| <code>read</code> | One volatile register read | Confirm GPIOE clock enable or inspect GPIO input |
| <code>write</code> | Construct and perform a volatile write | BSRR, which is stateless and write-only |
| <code>modify</code> | Volatile read followed by changed write | Configuration fields such as MODER |

<code>modify</code> is a read-modify-write sequence and is not atomic against another execution context. That is acceptable before interrupts exist and during one-time setup. BSRR is preferable for output changes because its set/reset operation is atomic per bit.

### Compare with the HAL

Open the source behind <code>set_high()</code> and <code>set_low()</code> with rust-analyzer's Go to Definition, or inspect the GPIO source on docs.rs.

Find that the HAL ultimately writes:

- <code>1 &lt;&lt; 1</code> to the set half of GPIOE BSRR for LD2 on;
- <code>1 &lt;&lt; (16 + 1)</code> to the reset half for LD2 off.

The HAL is not magic. It packages the same operation behind:

- exclusive pin ownership;
- a type proving that PE1 is an output;
- the correct port and pin at compile time;
- a small, reviewed unsafe implementation boundary.

### PAC checkpoint

Build and flash:

~~~bash
cargo check --bin register_lab
cargo build --bin register_lab
~~~

LD2 must blink through PAC field operations. Then reload <code>rover_launch</code> and confirm the higher-level game still works.

Your conclusion should be: “I know how the register write works, and I prefer the HAL unless I have a specific unsupported need.”

---

## Final acceptance run

### Static checks

Run only checks that actually complete:

~~~bash
cargo fmt --all -- --check
cargo check --bin blink
cargo check --bin rover_launch
cargo check --bin register_lab
cargo build --bin rover_launch
cargo build --bin register_lab
git diff --check
git status --short
~~~

### Physical checks

- [ ] Known-good blink still works.
- [ ] Released B1 reads LOW and pressed B1 reads HIGH on this board.
- [ ] Twenty presses produce twenty Pressed events.
- [ ] Holding B1 produces one Pressed event.
- [ ] Five normal game rounds complete without reset.
- [ ] A false start is detected.
- [ ] A timeout is detected and is not stored.
- [ ] No semihosting print occurs between GO and timestamp capture.
- [ ] Eight scores are retained.
- [ ] The ninth valid score replaces the oldest.
- [ ] Best and average are correct.
- [ ] Restarting the firmware constructs a fresh empty history.
- [ ] GDB shows the score data changing at a RAM address.
- [ ] The PAC lab toggles LD2 through BSRR.
- [ ] The game still works after the PAC lab is reloaded.

### Five-question debrief

Write short answers in a project note:

1. What is the difference between a GPIO level and a button edge?
2. What problem does debounce solve, and what latency did your algorithm add?
3. Where do the scoreboard, program instructions, and GPIO registers live?
4. Why is <code>wrapping_sub</code> appropriate for elapsed counters?
5. What safety and convenience does the HAL add over the PAC?

If you can answer those from your own code, the weekend succeeded.

---

## Troubleshooting ladder

| Symptom | First evidence to collect | Likely lesson |
| --- | --- | --- |
| New binary does not build | First exact compiler error | Resolve one ownership/type/configuration issue at a time |
| Link or PAC names look like H743 | Inspect the HAL feature in Cargo.toml | Device feature must match the confirmed STM32H753 silicon revision |
| <code>cargo run</code> mentions <code>$CHIP</code> | Inspect .cargo/config.toml | Probe-rs runner is unfinished; use OpenOCD/GDB |
| Button is always LOW | Mirror raw state and inspect SB51/SB58 | Verify physical mapping before inverting logic |
| One press counts several times | Log raw and stable states separately | Bounce needs filtering |
| A held button repeats | Inspect level-to-edge conversion | Events require remembered previous state |
| Reaction times are too large | Search for prints and breakpoints in Go | Semihosting and halts disturb time |
| DWT never advances | Check trace enable and counter-enabled bit | DCB trace must precede CYCCNT |
| Scores are “optimised out” in GDB | Confirm ELF and dev optimisation | Debug the matching opt-level-0 ELF |
| <code>defmt</code> text is absent in GDB | Identify transport and host decoder | RTT frames need an RTT-aware decoder |
| PAC and HAL fight over PE1 | Inspect ownership in one binary | One layer should own a peripheral at a time |

Keep a three-line debugging log:

~~~text
Observed:
Expected:
Next smallest test:
~~~

That habit is more valuable than guessing at several changes at once.

---

## Stretch goals, in order

Only start these after the finish-line checklist passes.

1. **Use red LD3 on PB14 for false starts.** This introduces a second GPIO port without changing the architecture.
2. **Show a score in LED pulses.** Encode hundreds and tens without needing a console.
3. **Add a controlled panic lab in its own binary.** Break on <code>rust_begin_unwind</code> and <code>HardFault</code>, inspect <code>bt</code>, then reload the game.
4. **Use TIM2 as a 1 kHz free-running application timer.** Compare it with the core-specific DWT counter.
5. **Replace polling with a PC13 EXTI interrupt.** The ISR should clear the pending bit and record only a flag/timestamp. It must not delay or print.
6. **Share interrupt state safely.** Start with <code>AtomicBool</code>/<code>AtomicU32</code>, then learn <code>critical-section</code> and a mutex.
7. **Report through ST-Link virtual COM port.** The stock board routes USART3 through PD8/PD9.
8. **Try the STM32 hardware RNG.** Compare real peripheral entropy with the human-timing seed.
9. **Extract pure game logic.** Test state transitions separately from GPIO.

### Do not make persistent flash the next quick stretch

Internal flash is not “RAM that survives reset.” STM32H753 flash requires erase-before-write rules, aligned flash-word programming, wear planning, and careful cache/power-loss handling. The current linker owns the entire 2 MiB flash range.

Before storing a high score, a future dedicated exercise must:

- reserve a sector explicitly in <code>memory.x</code>;
- prove the firmware image cannot occupy it;
- use the HAL flash/storage API;
- define a versioned, power-loss-tolerant record;
- consider erase wear;
- recover safely from an interrupted update.

An incorrect sector erase can erase firmware. RAM score history is the correct boundary for this weekend.

---

## Recommended learning path after this project

| Order | Concept | Concrete project | Crates or APIs to meet |
| --- | --- | --- | --- |
| 1 | External interrupts | B1 EXTI reaction capture | HAL EXTI, NVIC, atomics, <code>critical-section</code> |
| 2 | General-purpose timers | Move timing from DWT to TIM2 | HAL timer APIs, <code>fugit</code>, <code>nb</code> |
| 3 | Serial communication | Stream scores over ST-Link VCP | HAL serial, <code>core::fmt::Write</code> |
| 4 | Portable drivers | Add one I2C sensor | Check the driver's embedded-hal version |
| 5 | Fixed-capacity collections | Queue samples or format messages | <code>heapless</code> |
| 6 | Concurrency framework | Rebuild one known project | Choose RTIC **or** Embassy deliberately |
| 7 | DMA | Move peripheral data without CPU copies | DMA-accessible SRAM and Cortex-M7 cache rules |
| 8 | Persistent storage | Versioned high-score record | HAL flash, <code>embedded-storage</code>, linker reservation |

Embassy uses its own STM32 HAL stack, so treat it as a deliberate migration after the manual foundations—not as another crate to drop into this project.

The current linker places RAM at <code>0x20000000</code>, the DTCM region. Keep in mind for later that DMA1/2 cannot access ITCM/DTCM; DMA buffers will need a suitable SRAM bank plus Cortex-M7 cache-coherency planning.

## Primary references

- [ST UM2407: STM32H7 Nucleo-144 MB1364 user manual](https://www.st.com/resource/en/user_manual/um2407-stm32h7-nucleo144-board-stmicroelectronics.pdf)
- [ST RM0433: STM32H742/743/753/750 reference manual](https://www.st.com/resource/en/reference_manual/rm0433-stm32h743-753-and-stm32h750-value-line-advanced-arm-based-32-bit-mcus-stmicroelectronics.pdf)
- [stm32h7xx-hal 0.16.0 documentation](https://docs.rs/stm32h7xx-hal/0.16.0/stm32h7xx_hal/)
- [stm32h7xx-hal supported device features](https://github.com/stm32-rs/stm32h7xx-hal)
- [cortex-m DWT documentation](https://docs.rs/cortex-m/latest/cortex_m/peripheral/struct.DWT.html)
- [cortex-m-rt documentation](https://docs.rs/cortex-m-rt/latest/cortex_m_rt/)
- [Embedded Rust Book: memory-mapped registers](https://docs.rust-embedded.org/book/start/registers.html)
- [svd2rust register API](https://docs.rs/svd2rust/)
- [defmt printers and host decoders](https://defmt.ferrous-systems.com/printers)
- [semihosting crate documentation](https://docs.rs/semihosting/latest/semihosting/)

Local companion notes:

- <code>ai_summary/blink.md</code>
- <code>ai_summary/embedded-rust-gdb-semihosting.md</code>

Have fun with it. The satisfying moment is not the first score—it is being able to trace that score from a physical switch, through a GPIO register and a Rust state transition, into an array you can see changing in RAM.
