I’d keep the same three, but give each a different focus: **simulation and performance; concurrency and reliable data handling; computer architecture and interpreters.** Each learning point should correspond to something you build, test or compare—not just another library you install.

## 1. Falling-sand laboratory

*Build a sandbox where you paint sand, water and walls, then experiment with their behaviour.*

**First version:** Sand falls and piles up against walls. You can paint, pause and reset.

The broader learning points:

* **Simulation models and update order.** Define local movement rules, then investigate what happens when you update cells from top to bottom versus bottom to top. Compare updating the grid in place with reading an “old” grid and writing a “new” one—double buffering. For moving particles, you must also resolve conflicts when two particles choose the same destination; separate buffers alone do not solve that. ([Game Programming Patterns][1])
* **Data representation and memory layout.** Store the grid in a flat `Vec<Cell>`, using `index = y * width + x`. A Rust vector stores its elements contiguously. Use this as an opportunity to understand the relationship between your logical two-dimensional world and its actual representation in memory. ([Rust Documentation][2])
* **Algorithm design and performance measurement.** Start by checking every cell each tick. Later, experiment with tracking active regions so settled areas can be skipped. Benchmark both approaches on mostly empty, settled and highly active worlds rather than assuming the more complicated algorithm wins.
* **Time, determinism and testing.** Separate simulation ticks from rendering frames. Record mouse actions against tick numbers and control any randomness so you can replay an experiment. Fixed-step game loops provide a useful foundation for this separation. ([Game Programming Patterns][3])

**Rust practice:** Enums, collections, borrowing, separating mutable state from update logic, and tests.

**A useful challenge:** With painting disabled and boundaries closed, run thousands of updates and verify that no sand or water is created or destroyed. Replay the same inputs and verify the final grid matches.

**Finish at:** Sand, water, walls, replay and one measured optimisation. Leave realistic fluid physics and GPU programming out.

## 2. Home-sensor mission control

*Build a dashboard for your future temperature and humidity sensors—but deliberately give it unreliable simulated devices first.*

**First version:** Three simulated sensors feed readings into a terminal dashboard with current values and connection status.

The broader learning points:

* **Concurrency and message-passing architecture.** Give each sensor a producer task and send readings through a bounded queue to a task that owns the application state. Deliberately slow the consumer and observe what happens when the queue fills. This teaches backpressure: controlling how producers behave when consumers cannot keep up. Tokio documents this pattern directly. ([Tokio][4])
* **Reliability and idempotency.** Inject duplicates, delayed messages and disconnects. Give each reading a stable identifier that stays unchanged during retries, and ensure processing it twice does not create two stored readings. Implement that rule with a database uniqueness constraint and explicit conflict handling. ([SQLite][5])
* **Database design and query performance.** Store readings in SQLite and implement “show this sensor’s last 24 hours.” Compare the query before and after adding an index on sensor ID and timestamp. Investigate the query plan, not just the elapsed time; SQLite’s documentation explains how multi-column indexes support searching and ordering. ([SQLite][6])
* **Dependency injection and testable design.** Make the reading source and clock replaceable inputs to your core logic. Run the same application with simulated readings, a recorded session or a real sensor. In tests, advance a fake clock to check stale-device detection without waiting in real time.

**Rust practice:** `async`/`.await`, channels, traits, serialisation, typed errors and ownership across tasks.

**A useful challenge:** Replay a batch with every reading duplicated and shuffled. Verify that storage contains one copy of each reading and that an old message cannot replace a newer value on the dashboard.

**Finish at:** Simulated sensors, bounded queues, local history and failure tests. Add real hardware afterwards; do not build a cloud platform.

## 3. CHIP-8 emulator with a debugger

*Build a small virtual machine that runs games, then expose enough of its internals to see how programs execute.*

**First version:** Load a program and execute enough instructions to display a test image.

The broader learning points:

* **Computer architecture.** Implement memory, registers, a program counter and a call stack. Follow the fetch–decode–execute cycle and observe how jumps and subroutine calls change execution. This is an interpreter for a virtual instruction set, rather than an electrical simulation of a physical processor. ([Tvil][7])
* **Interpreter design and separation of concerns.** Decode instruction bytes into an `Instruction` enum, then execute that representation in a separate function. Reuse the decoder to build a disassembler that displays readable instructions. Your debugger and execution engine should not maintain separate, inconsistent interpretations of the same bytes.
* **Timing and state machines.** Represent running, paused and waiting-for-input states explicitly. Keep instruction execution separate from the delay and sound timers: CHIP-8’s timers tick at 60 Hz independently of the instruction loop’s speed. ([Tvil][7])
* **Debugging and conformance testing.** Add single-step execution, breakpoints and a register inspector. Write small instruction-level tests, then use the Timendus test suite for broader checks of instructions, arithmetic flags, input and compatibility behaviour. ([GitHub][8])

**Rust practice:** Pattern matching, bit manipulation, arrays, slices, explicit integer behaviour and error handling.

**A useful challenge:** Pause a running game, predict what the next instruction will change, step once and compare your prediction with the machine state.

**Finish at:** A working game, instruction tests and a useful debugger. Leave additional instruction-set extensions and an assembler for later.

I’d still start with the **falling-sand laboratory**. Across all three, use the same learning cycle: build the simplest working approach, expose a limitation, then improve it and demonstrate the difference with a test or measurement.

[1]: https://gameprogrammingpatterns.com/double-buffer.html?utm_source=chatgpt.com "Double Buffer - Game Programming Patterns"
[2]: https://doc.rust-lang.org/std/vec/struct.Vec.html "Vec in std::vec - Rust"
[3]: https://gameprogrammingpatterns.com/game-loop.html?utm_source=chatgpt.com "Game Loop · Sequencing Patterns"
[4]: https://tokio.rs/tokio/tutorial/channels "Channels | Tokio - An asynchronous Rust runtime"
[5]: https://sqlite.org/lang_upsert.html?utm_source=chatgpt.com "UPSERT"
[6]: https://sqlite.org/queryplanner.html?utm_source=chatgpt.com "Query Planning"
[7]: https://tobiasvl.github.io/blog/write-a-chip-8-emulator/ "Guide to making a CHIP-8 emulator | Tvil"
[8]: https://github.com/Timendus/chip8-test-suite "GitHub - Timendus/chip8-test-suite: A collection of ROM images with tests that will aid you in developing your own CHIP-8, SUPER-CHIP or XO-CHIP interpreter (or \"emulator\") · GitHub"

