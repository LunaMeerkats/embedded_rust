**A window is created by asking the operating system’s windowing system for one, then processing the events it sends back.** On Windows, the native approach is Win32: register a window class, create the window, and run a message loop. You are not drawing the title bar and implementing mouse input yourself. ([Microsoft Learn][1])

For your Rust project, I recommend starting **without a game engine, using `winit` as the windowing layer**. It handles platform-specific window creation and events, but does not draw your graphics or run your simulation. This is not a zero-library implementation; it is a useful boundary for learning application structure without first writing Windows-specific bindings. ([GitHub][2])

## Create a minimal window

In your terminal:

```bash
cargo new sand_lab
cd sand_lab
cargo add winit@0.30.13
```

This example uses the `winit` 0.30 API, with `ApplicationHandler` and `run_app`. ([Docs.rs][3])

Replace `src/main.rs` with:

```rust
use std::error::Error;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

// Our application owns its window once it has been created.
#[derive(Default)]
struct App {
    window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // This callback can happen more than once.
        if self.window.is_some() {
            return;
        }

        // Describe the window. This does not create it yet.
        let attributes = Window::default_attributes()
            .with_title("Sand Lab")
            .with_inner_size(LogicalSize::new(800.0, 600.0));

        // Ask the windowing system to create it.
        let window = event_loop
            .create_window(attributes)
            .expect("Failed to create the window");

        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                println!("Resized to {} × {} pixels", size.width, size.height);
            }

            // Ignore other events for now.
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;

    // Wait for events instead of continuously spinning.
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
```

Run it:

```bash
cargo run
```

Try resizing the window and watch the terminal output. We have not implemented rendering, so the window’s interior has no defined appearance yet—do not expect a particular background colour. `winit` explicitly leaves drawing to another layer. ([Docs.rs][4])

## What the important pieces do

**`App` stores state that must survive between events.**

In this example, that state is the window. Initially, `window` is `None`; after creation, we assign `Some(window)`. Later, this same struct can also own your simulation grid, mouse position and pause state.

The `Window` value represents and manages a native window; it is not an array of the pixels displayed inside it. On desktop platforms, dropping this value closes the corresponding window, which is why we retain it rather than leave it as a temporary local variable. ([Docs.rs][5])

**`ApplicationHandler` defines how your application responds.**

The `impl ApplicationHandler for App` block supplies the callbacks that `winit` invokes. You do not call `resumed` or `window_event` yourself.

This is an example of **inversion of control**: rather than your code repeatedly asking whether something happened, you give the event system methods to call when something happens. The required callbacks here are `resumed` and `window_event`. ([Docs.rs][3])

**`resumed` is where the window is created.**

The name can look odd at startup. `winit` sends an initial resume event even on platforms without a formal suspend/resume lifecycle. It recommends creating windows after that event for portability. The `is_some()` check prevents a second resume event from creating another window. ([Docs.rs][3])

Notice the separation between **describing a resource** and **creating it**: `Window::default_attributes()` produces configuration; `create_window(attributes)` performs creation. ([Docs.rs][5])

**`run_app` runs the event loop.**

It dispatches events to your callbacks until you request an exit. `ControlFlow::Wait` lets the event loop wait when nothing needs processing rather than repeatedly checking in a busy loop. This first version needs neither `async` nor an extra thread. ([Docs.rs][4])

The key distinction for the sand project is: **windowing receives events, simulation changes your world, and rendering turns that world into pixels.** Keep those responsibilities separate from the beginning.

[1]: https://learn.microsoft.com/en-us/windows/win32/learnwin32/your-first-windows-program "Module 1. Your First Windows Program - Win32 apps | Microsoft Learn"
[2]: https://github.com/rust-windowing/winit?utm_source=chatgpt.com "Winit - Window handling library in pure Rust · GitHub"
[3]: https://docs.rs/winit/latest/winit/application/trait.ApplicationHandler.html?utm_source=chatgpt.com "ApplicationHandler in winit::application - Rust - Docs.rs"
[4]: https://docs.rs/winit/latest/x86_64-apple-ios/winit/ "winit - Rust"
[5]: https://docs.rs/winit/latest/winit/window/struct.Window.html?utm_source=chatgpt.com "Window in winit::window - Rust"

