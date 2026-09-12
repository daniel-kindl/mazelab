# ratatui and crossterm: versions and API shape

Research for [issue #2](https://github.com/daniel-kindl/mazelab/issues/2). Map: [issue #1](https://github.com/daniel-kindl/mazelab/issues/1).

Verified on 2026-09-12 against crates.io, docs.rs, and the upstream repositories. No blog post or tutorial is used as a source.

There is no existing convention for research notes in this repo. This file starts `docs/research/`.

## Recommendation

Pin this pair:

```toml
[dependencies]
ratatui = "0.30.2"     # default features; selects crossterm 0.29 internally
crossterm = "0.29"     # matches the version inside ratatui
```

`ratatui` 0.30.2 is the current stable release. `crossterm` 0.29.0 is the current stable release. `ratatui` 0.30.2 with default features selects the crossterm 0.29.x backend, so Cargo unifies both requirements on crossterm 0.29.0.

The project MSRV becomes **1.88.0**, set by `ratatui` 0.30.2. Use this value in the CI matrix.

---

## 1. Current releases and the compatible pair

| Crate | Version | Released | `rust-version` |
| --- | --- | --- | --- |
| `ratatui` | 0.30.2 | 2026-06-19 | 1.88.0 |
| `crossterm` | 0.29.0 | 2025-04-05 | 1.63.0 |
| `ratatui-crossterm` | 0.1.2 | 2026-06-19 | 1.88.0 |

Sources: `max_stable_version` and version metadata at <https://crates.io/api/v1/crates/ratatui> and <https://crates.io/api/v1/crates/crossterm>; the published manifests at <https://docs.rs/crate/ratatui/0.30.2/source/Cargo.toml> and <https://docs.rs/crate/crossterm/0.29.0/source/Cargo.toml>.

### ratatui no longer depends on crossterm directly

Since 0.30.0 the project is a workspace. The crate root documents the layout:

> Starting with Ratatui 0.30.0, the project was reorganized into a modular workspace... `ratatui`: Main crate with complete functionality (recommended for apps); `ratatui-core`: Core traits and types for widget libraries; `ratatui-widgets`: Built-in widget implementations; Backend crates: `ratatui-crossterm`, `ratatui-termion`, `ratatui-termina`, `ratatui-termwiz`; `ratatui-macros`

Source: <https://docs.rs/ratatui/0.30.2/ratatui/>

`ratatui` 0.30.2 has no normal dependency on `crossterm`. It depends on `ratatui-crossterm` 0.1.2, which holds the crossterm dependency. From the published manifests:

```toml
# ratatui 0.30.2
[features]
crossterm = ["dep:ratatui-crossterm", "std"]
crossterm_0_28 = ["crossterm", "ratatui-crossterm/crossterm_0_28"]
crossterm_0_29 = ["crossterm", "ratatui-crossterm/crossterm_0_29"]
default = ["all-widgets", "crossterm", "layout-cache", "macros", "underline-color"]
```

```toml
# ratatui-crossterm 0.1.2
[features]
crossterm_0_28 = ["dep:crossterm_0_28"]
crossterm_0_29 = ["dep:crossterm_0_29"]
default = ["crossterm_0_29", "underline-color"]

[dependencies.crossterm_0_28]
version = "0.28"
optional = true
package = "crossterm"

[dependencies.crossterm_0_29]
version = "0.29"
optional = true
package = "crossterm"
```

Sources: <https://docs.rs/crate/ratatui/0.30.2/source/Cargo.toml>, <https://docs.rs/crate/ratatui-crossterm/0.1.2/source/Cargo.toml>

The supported range is crossterm 0.28.x **or** 0.29.x, chosen by feature flag. 0.29 is the default. `ratatui-crossterm` documents the policy:

> Ratatui will support at least the two most recent versions of Crossterm... The highest enabled feature flag of the available `crossterm_0_xx` features... takes precedence.

Source: <https://docs.rs/crate/ratatui-crossterm/0.1.2/source/src/lib.rs>

### The re-export

`ratatui` re-exports the selected crossterm crate. From the published source:

```rust
/// re-export the `crossterm` crate so that users don't have to add it as a dependency
#[cfg(feature = "crossterm")]
pub use ratatui_crossterm::crossterm;
```

Source: <https://docs.rs/crate/ratatui/0.30.2/source/src/lib.rs>

`ratatui-crossterm` documents the re-export as the way to avoid a version conflict:

> To promote interoperability within the Ratatui ecosystem, the selected Crossterm crate is re-exported as `ratatui_crossterm::crossterm`. This re-export is essential for authors of widget libraries or any applications that need to perform direct Crossterm operations while ensuring compatibility with the version used by `ratatui-crossterm`... developers can avoid version conflicts

Source: <https://docs.rs/crate/ratatui-crossterm/0.1.2/source/src/lib.rs>

The `ratatui` Quickstart shows the other choice, a direct dependency: "Add `ratatui` and `crossterm` as dependencies to your cargo.toml: `cargo add ratatui crossterm`", and its examples write `use crossterm::event::{...}`. Source: <https://docs.rs/ratatui/0.30.2/ratatui/>

**Guidance for MazeLab:** declare `crossterm = "0.29"` as the Quickstart shows, and import through `ratatui::crossterm` in application code. The import path removes the risk that a future `cargo update` moves the two apart.

### What MazeLab declares

Declare `ratatui` only. The crate root documents this:

> For application developers: `ratatui` remains the recommended starting point. For widget library authors: Consider depending on `ratatui-core` instead of the full `ratatui` crate for better API stability and reduced dependencies.

Reach for `ratatui-crossterm` directly only "when your application needs backend-specific APIs or a direct backend dependency". Source: <https://docs.rs/ratatui/0.30.2/ratatui/>

The sub-crates carry independent version numbers. `ratatui-core` is 0.1.2, not 0.30.x.

---

## 2. Terminal setup and teardown

### `ratatui::run` is the documented pattern

`ratatui::run` is new in 0.30.0. The Quickstart uses it:

```rust
use crossterm::event;

fn main() -> std::io::Result<()> {
    ratatui::run(|mut terminal| {
        loop {
            terminal.draw(|frame| frame.render_widget("Hello World!", frame.area()))?;
            if event::read()?.is_key_press() {
                break Ok(());
            }
        }
    })
}
```

Source: <https://docs.rs/ratatui/0.30.2/ratatui/>

The crate root section "Which setup path should I use?" states:

> - Use `run()` for normal applications. It initializes the terminal, runs your app, and restores the terminal on exit.
> - Use `init()` / `restore()` (or `try_init()` / `try_restore()`) when you want manual control over terminal lifetime and the event loop structure.
> - Use `init_with_options()` / `try_init_with_options()` when you need a custom `Viewport` [...]

### Signatures

| Item | Signature |
| --- | --- |
| `ratatui::run` | `pub fn run<F, R>(f: F) -> R where F: FnOnce(&mut DefaultTerminal) -> R` |
| `ratatui::init` | `pub fn init() -> DefaultTerminal` |
| `ratatui::try_init` | `pub fn try_init() -> Result<DefaultTerminal>` |
| `ratatui::restore` | `pub fn restore()` |
| `DefaultTerminal` | `pub type DefaultTerminal = Terminal<CrosstermBackend<Stdout>>;` |
| `Terminal::new` | `pub fn new(backend: B) -> Result<Terminal<B>, <B as Backend>::Error>` |
| `CrosstermBackend::new` | `pub const fn new(writer: W) -> CrosstermBackend<W>` |

Sources: <https://docs.rs/ratatui/0.30.2/ratatui/fn.run.html>, <https://docs.rs/ratatui/0.30.2/ratatui/fn.init.html>, <https://docs.rs/ratatui/0.30.2/ratatui/fn.restore.html>, <https://docs.rs/ratatui/0.30.2/ratatui/type.DefaultTerminal.html>, <https://docs.rs/ratatui/0.30.2/ratatui/struct.Terminal.html>, <https://docs.rs/ratatui/0.30.2/ratatui/backend/struct.CrosstermBackend.html>

The four `init` and `restore` functions are gated on the `crossterm` feature. `ratatui::init` returns `DefaultTerminal`, not a `Result`.

`ratatui::terminal` is a private module. The type path is `ratatui::Terminal`.

### What `init` documents

> This will create a new `DefaultTerminal` and initialize it with the following defaults:
>
> - Backend: `CrosstermBackend` writing to `Stdout`
> - Raw mode is enabled
> - Alternate screen buffer enabled
> - A panic hook is installed that restores the terminal before panicking. Ensure that this method is called after any other panic hooks that may be installed to ensure that the terminal is restored before those hooks are called.

Source: <https://docs.rs/ratatui/0.30.2/ratatui/fn.init.html>

The `init` module states the same guarantee for every initialization function:

> **Panic Hook** — All initialization functions install a panic hook that automatically restores the terminal state before panicking. This ensures that even if your application panics, the terminal will be left in a usable state.
>
> **Important**: Call the initialization functions after installing any other panic hooks to ensure the terminal is restored before other hooks run.

Source: <https://docs.rs/ratatui/0.30.2/ratatui/init/index.html>

`Terminal::new` documents the opposite:

> Note that unlike `ratatui::init`, this does not install a panic hook, so it is recommended to do that manually when using this function, otherwise any panic messages will be printed to the alternate screen and the terminal may be left in an unusable state.

Source: <https://docs.rs/ratatui/0.30.2/ratatui/struct.Terminal.html#method.new>

### What `restore` documents

> This function will attempt to restore the terminal to its original state by performing the following steps:
>
> - Raw mode is disabled.
> - The alternate screen buffer is left.
>
> If either of these steps fail, the error is printed to stderr and ignored.

Source: <https://docs.rs/ratatui/0.30.2/ratatui/fn.restore.html>

### Comparison table from the `init` module docs

| Function | Alternate Screen | Raw Mode | Error Handling | Use Case |
| --- | --- | --- | --- | --- |
| `run` | yes | yes | Auto-cleanup | Simple apps |
| `init` | yes | yes | Panic | Standard full-screen apps |
| `try_init` | yes | yes | Result | Standard apps with error handling |
| `init_with_options` | no | yes | Panic | Custom viewport apps |
| `try_init_with_options` | no | yes | Result | Custom viewport with error handling |

Source: <https://docs.rs/ratatui/0.30.2/ratatui/init/index.html>

The `*_with_options` variants do **not** enter the alternate screen. MazeLab wants the alternate screen, so it must not use them.

### crossterm items, for the manual path

| Item | Signature |
| --- | --- |
| `enable_raw_mode` | `pub fn enable_raw_mode() -> Result<()>` |
| `disable_raw_mode` | `pub fn disable_raw_mode() -> Result<()>` |
| `EnterAlternateScreen` | `pub struct EnterAlternateScreen;` |
| `LeaveAlternateScreen` | `pub struct LeaveAlternateScreen;` |
| `execute!` | `macro_rules! execute { ($writer:expr $(, $command:expr)* $(,)? ) => { ... }; }` |

`crossterm::Result<T>` is `std::io::Result<T>`. Both alternate-screen structs document: "Commands must be executed/queued for execution otherwise they do nothing."

Sources: <https://docs.rs/crossterm/0.29.0/crossterm/terminal/index.html>, <https://docs.rs/crossterm/0.29.0/crossterm/macro.execute.html>

---

## 3. Input and frame timing

### Signatures

```rust
pub fn poll(timeout: Duration) -> Result<bool>   // std::io::Result<bool>
pub fn read() -> Result<Event>                   // std::io::Result<Event>
```

`poll` documents: "Returns `Ok(true)` if an `Event` is available otherwise it returns `Ok(false)`. `Ok(true)` guarantees that subsequent call to the `read` function won't block."

`read` documents: "This function blocks until an `Event` is available. Combine it with the `poll` function to get non-blocking reads."

The module documents a threading constraint:

> It's not allowed to call these functions from different threads or combine them with the `EventStream`. You're allowed to either: use the `read` & `poll` functions on any, but same, thread — or the `EventStream`.

It also documents: "Make sure to enable raw mode in order for keyboard events to work properly".

Sources: <https://docs.rs/crossterm/0.29.0/crossterm/event/index.html>, <https://docs.rs/crossterm/0.29.0/crossterm/event/fn.poll.html>, <https://docs.rs/crossterm/0.29.0/crossterm/event/fn.read.html>

### The `Event` enum

```rust
pub enum Event {
    FocusGained,
    FocusLost,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
}
```

Mouse and focus events are off by default: "You have to enable them with the `EnableMouseCapture` / `EnableFocusChange` command."

`Event` carries inherent helpers that current official examples use in place of a hand-written `match`: `is_key_press`, `is_key_release`, `is_key_repeat`, `is_resize`, `as_key_press_event`, `as_resize_event`, and more.

Source: <https://docs.rs/crossterm/0.29.0/crossterm/event/enum.Event.html>

### `KeyEvent` and the Windows caveat

```rust
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub kind: KeyEventKind,
    pub state: KeyEventState,
}

pub enum KeyEventKind { Press, Repeat, Release }
```

The `kind` field documents when it is set:

> Only set if:
> - Unix: `KeyboardEnhancementFlags::REPORT_EVENT_TYPES` has been enabled with `PushKeyboardEnhancementFlags`.
> - Windows: always

Source: <https://docs.rs/crossterm/0.29.0/crossterm/event/struct.KeyEvent.html>

crossterm's own API docs do not state the duplicate-event consequence. The ratatui FAQ does:

> However, on Windows, when using `Crossterm`, this will send the same `Event::Key(e)` twice; one for when you press the key, i.e. `KeyEventKind::Press` and one for when you release the key, i.e. `KeyEventKind::Release`. On `MacOS` and `Linux` only `KeyEventKind::Press` kinds of `key` event is generated.

Source: <https://raw.githubusercontent.com/ratatui/ratatui-website/main/src/content/docs/faq.md>

**Guidance for MazeLab:** filter on `KeyEventKind::Press`, or call `Event::as_key_press_event()`. Without the filter every key acts twice on Windows. This matters because most MazeLab keys toggle or step.

The crate root also states: "Ratatui does not include any input handling." Source: <https://docs.rs/ratatui/0.30.2/ratatui/>

### Poll with timeout against a fixed tick

An official example shows the pattern. The file is byte-identical on the `main` and `latest` branches, so it matches released 0.30.2.

```rust
fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>>
where
    B::Error: 'static,
{
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| ui::render(frame, &mut app))?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if !event::poll(timeout)? {
            app.on_tick();
            last_tick = Instant::now();
            continue;
        }
        if let Some(key) = event::read()?.as_key_press_event() {
            match key.code {
                KeyCode::Char('h') | KeyCode::Left => app.on_left(),
                KeyCode::Char(c) => app.on_key(c),
                _ => {}
            }
        }
        if app.should_quit {
            return Ok(());
        }
    }
}
```

Source: <https://raw.githubusercontent.com/ratatui/ratatui/latest/examples/apps/demo/src/crossterm.rs>

The shape: `timeout = tick_rate.saturating_sub(last_tick.elapsed())`; `poll` false means tick and reset `last_tick`; `poll` true means one `read`. `draw` runs at the top of every iteration.

A second example uses a plain fixed timeout:

```rust
// Ensure that the app only blocks for a period that allows the app to render at
// approximately 60 FPS (this doesn't account for the time to render the frame, and will
// also update the app immediately any time an event occurs)
let timeout = Duration::from_secs_f32(1.0 / 60.0);
if !event::poll(timeout)? {
    return Ok(());
}
```

Source: <https://raw.githubusercontent.com/ratatui/ratatui/latest/examples/apps/colors-rgb/src/main.rs>

The comment names the limitation. The timeout does not account for render time, and any input event ends the wait early.

**Guidance for MazeLab (feeds issue #7):** the loop above does not deliver a constant number of ticks per second. Input makes it iterate more often. Map decision 11 defines a fractional step budget against a render tick near 60 Hz. Derive the budget from measured elapsed time, not from a count of loop iterations. Otherwise a user who holds a key down speeds the animation up.

There is no prose recipe page for tick rate on the ratatui website. The two examples above are the primary sources.

---

## 4. Resize

**The app does not have to handle `Event::Resize` to get correct layout.** Read `Frame::area()` inside the draw callback.

The crate root documents the behavior:

> **If the terminal is resized:** Ratatui does not redraw automatically when a resize event arrives. Your app should continue the event loop and call `Terminal::draw` again. During that render pass, Ratatui checks the backend's current size instead of assuming the resize events were complete or up to date. This keeps layout based on the size that actually exists when rendering, even if multiple resize events were coalesced, missed, or delivered before the UI redraws. Fullscreen and inline viewports update their internal size during that render pass; fixed viewports keep their configured rectangle until you call `Terminal::resize`.

Source: <https://docs.rs/ratatui/0.30.2/ratatui/>

`Terminal::autoresize` carries the guarantee:

```rust
pub fn autoresize(&mut self) -> Result<(), <B as Backend>::Error>
```

> Queries the backend for size and resizes if it doesn't match the previous size.
>
> This is called automatically during `Terminal::draw` / `Terminal::try_draw` for fullscreen and inline viewports. Fixed viewports are not automatically resized.
>
> If the size changed, this calls `Terminal::resize` and therefore clears the affected region before the next frame is rendered.

Source: <https://docs.rs/ratatui/0.30.2/ratatui/struct.Terminal.html#method.autoresize>

The `Terminal` page states how to treat a resize event:

> If your event loop receives a resize event, treat that event as a signal to render again rather than as a complete source of truth for layout. During a render pass, use `Frame::area` as the rectangle that Ratatui has actually prepared for drawing.

`Frame::area` repeats it:

> Returns the area of the current frame. This is guaranteed not to change during rendering, so may be called multiple times.
>
> If your app listens for a resize event from the backend, ignore that event's dimensions for calculations performed during the current render callback and use this value instead.

Sources: <https://docs.rs/ratatui/0.30.2/ratatui/struct.Terminal.html>, <https://docs.rs/ratatui/0.30.2/ratatui/struct.Frame.html#method.area>

The official resize example leaves the arm empty on purpose:

```rust
Event::Resize(_, _) => {
    // The next `draw` pass re-renders the UI at the new size.
}
```

Source: <https://docs.rs/ratatui/0.30.2/ratatui/init/index.html>

Related signatures:

```rust
pub fn resize(&mut self, area: Rect) -> Result<(), <B as Backend>::Error>
pub fn size(&self) -> Result<Size, <B as Backend>::Error>   // returns Size, not Rect
pub const fn area(&self) -> Rect                             // Frame::area
pub const fn size(&self) -> Rect                             // Frame::size, deprecated
```

`Terminal::size` documents: "This returns the backend's current terminal size and does not update Ratatui's internal viewport bookkeeping by itself."

`Frame::size` is deprecated but still present in 0.30.2. The banner reads "use `area()` instead".

**Guidance for MazeLab (map decision 9):** MazeLab uses the fullscreen viewport and already redraws on a ~60 Hz tick, so it needs no `Event::Resize` arm. Compare `Frame::area()` against the footprint `4W+2` by `2H+1` inside the draw callback. If the area is too small, render the "terminal too small" panel. The maze is unchanged, as decision 9 requires.

---

## 5. `TestBackend` for snapshot tests

```rust
pub fn new(width: u16, height: u16) -> TestBackend
pub fn with_lines<'line, Lines>(lines: Lines) -> TestBackend
    where Lines: IntoIterator, <Lines as IntoIterator>::Item: Into<Line<'line>>
pub const fn buffer(&self) -> &Buffer
pub const fn cursor_position(&self) -> Position
pub fn resize(&mut self, width: u16, height: u16)
pub fn assert_buffer(&self, expected: &Buffer)
pub fn assert_buffer_lines<'line, Lines>(&self, expected: Lines)
    where Lines: IntoIterator, <Lines as IntoIterator>::Item: Into<Line<'line>>
pub fn assert_cursor_position<P>(&mut self, position: P) where P: Into<Position>
```

On `Terminal`:

```rust
pub const fn backend(&self) -> &B
pub const fn backend_mut(&mut self) -> &mut B
```

Documented semantics:

- `assert_buffer` — "Asserts that the `TestBackend`'s buffer is equal to the expected buffer. This is a shortcut for `assert_eq!(self.buffer(), &expected)`." It panics with "a detailed error message showing the differences".
- `assert_buffer_lines` — "Asserts that the `TestBackend`'s buffer is equal to the expected lines. This is a shortcut for `assert_eq!(self.buffer(), &Buffer::with_lines(expected))`."
- `with_lines` — "Creates a new `TestBackend` with the specified lines as the initial screen state. The backend's screen size is determined from the initial lines."

`Buffer::with_lines`:

```rust
pub fn with_lines<'a, Iter>(lines: Iter) -> Buffer
    where Iter: IntoIterator, <Iter as IntoIterator>::Item: Into<Line<'a>>
```

The item bound is `Into<Line>`, so an array of `&str` works. The struct-level example:

```rust
use ratatui::backend::{Backend, TestBackend};

let mut backend = TestBackend::new(10, 2);
backend.clear()?;
backend.assert_buffer_lines(["          "; 2]);
```

Sources: <https://docs.rs/ratatui/0.30.2/ratatui/backend/struct.TestBackend.html>, <https://docs.rs/ratatui/0.30.2/ratatui/buffer/struct.Buffer.html#method.with_lines>

### `assert_buffer_eq!` is gone from the `ratatui` facade

`ratatui::assert_buffer_eq` does not exist in 0.30.2. The macro page returns 404 and the crate's item index lists one macro only, `border`. The macro still exists in `ratatui-core` 0.1.2, marked deprecated with "use `assert_eq!(&actual, &expected)`".

Source: <https://docs.rs/ratatui-core/0.1.2/ratatui_core/macro.assert_buffer_eq.html>

| API | Status in 0.30.2 |
| --- | --- |
| `TestBackend::new(w, h)` | current |
| `TestBackend::buffer()` | current |
| `TestBackend::assert_buffer(&Buffer)` | current |
| `TestBackend::assert_buffer_lines([...])` | current, preferred |
| `Buffer::with_lines([...])` | current |
| `Terminal::backend()` | current |
| `assert_buffer_eq!` | deprecated, and not reachable through `ratatui` |

The struct documents a scope caveat:

> Note: that although many of the integration and unit tests in ratatui are written using this backend, it is preferable to write unit tests for widgets directly against the buffer rather than using this backend. This backend is intended for integration tests that test the entire terminal UI.

**Guidance for MazeLab (map decision 20):** use `assert_buffer_lines` for the snapshot tier. Test the display-grid derivation and the priority order against a `Buffer` directly where a full terminal is not needed.

---

## 6. Breaking changes that invalidate older code

Source for this section: <https://raw.githubusercontent.com/ratatui/ratatui/main/BREAKING-CHANGES.md> and <https://raw.githubusercontent.com/ratatui/ratatui/main/CHANGELOG.md>

### The last two ratatui releases

**0.30.2 (2026-06-19)** — no entry in `BREAKING-CHANGES.md`. The changelog section holds Features, Bug Fixes, and Miscellaneous Tasks only.

**0.30.1 (2026-06-05)** — two documented items:

- "MSRV is now 1.88.0."
- "Adding `AsRef` impls for widgets may affect type inference (#2297): Adding `AsRef<Self>` for built-in widgets can change type inference outcomes in rare cases where `AsRef` is part of a trait bound, and can also conflict with downstream blanket or manual `AsRef` impls for widget types. If you hit new ambiguity errors, add explicit type annotations..."

0.30.1 also reintroduced `Cell::skip` as a deprecated field "in order to avoid breaking the API in a patch release" (#2437).

### 0.30.0: the release that invalidates an older tutorial

`Backend` now has an associated `Error` type (#1778). This changes the return type of nearly every `Terminal` method:

```rust
pub fn draw<F>(&mut self, render_callback: F)
    -> Result<CompletedFrame<'_>, <B as Backend>::Error>
    where F: FnOnce(&mut Frame<'_>)
```

The prescribed migration:

```diff
- fn run<B: Backend>(mut terminal: Terminal<B>) -> io::Result<()> {
+ fn run<B: Backend>(mut terminal: Terminal<B>) -> Result<(), B::Error> {
```

`CrosstermBackend` sets `type Error = std::io::Error`, so code that uses `DefaultTerminal` and returns `std::io::Result<()>` still compiles. Only helpers generic over `B` need edits. The official demo needed `where B::Error: 'static` to box the error.

`TestBackend` uses `core::convert::Infallible` as its `Error` (#1823). A test helper typed `-> io::Result<()>` will not compile against a `Terminal<TestBackend>`.

Other 0.30.0 items:

- Custom `Backend` impls must implement `clear_region`. It has no default implementation.
- `Flex::SpaceAround` now mirrors flexbox. Use `Flex::SpaceEvenly` for the old behavior.
- `block::Title` no longer exists. `block::Position` became `widgets::TitlePosition`. `Block::title()` takes `Into<Line>`.
- `Style` no longer implements `Styled`.
- `layout::Alignment` renamed to `layout::HorizontalAlignment`.
- `List::highlight_symbol` takes `Into<Line>` instead of `&str`.
- `Marker` is now `#[non_exhaustive]`. `symbols::braille::BLANK` and `DOTS` are removed.
- Disabling `default-features` also disables the layout cache, which costs performance.
- MSRV was raised to 1.86.0, then to 1.88.0 in 0.30.1.

Older items that still catch tutorial code:

- 0.28.0: `ratatui::terminal` is a private module.
- 0.28.0: `Frame::size` is deprecated and renamed to `Frame::area`.
- 0.28.0: `Backend::size` returns `Size` instead of `Rect`.

Not released, listed under `v0.31.0` in `BREAKING-CHANGES.md`: "`Backend` adds cursor save and restore methods". It affects custom backends only.

The documentation guidance also shifted. The crate root now steers a first-time reader to `ratatui::run`. `Terminal::new` with manual `enable_raw_mode` and `execute!` is framed as the advanced path: "Reach for `Terminal::new` or `Terminal::with_options` directly only when you need custom backend construction or terminal setup that Ratatui's convenience functions do not manage."

### The last two crossterm releases

Source: <https://raw.githubusercontent.com/crossterm-rs/crossterm/master/CHANGELOG.md>

- **0.29.0 (2025-04-05)** — Breaking: "Correctly fix `KeyModifiers` Display impl. Properly adding `+` in between modifiers (#979)".
- **0.28.1 (2024-08-01)** — Breaking: "Fix desync with mio and signalhook between repo and published crate. (upgrade to mio 1.0)". crossterm 0.28.0 is yanked on crates.io.

Unreleased on `master`, and therefore not in 0.29.0: MSRV raised from 1.63 to 1.85, and the `IsTty` trait removed in favor of `std::io::IsTerminal`. Expect both when 0.30 ships.

---

## Notes against the map (issue #1)

**Decision 18 (a `Drop` guard plus a chained panic hook).** `ratatui::init` and `ratatui::run` already document that they install a panic hook that restores the terminal. `ratatui::restore` documents that it disables raw mode and leaves the alternate screen. So the standing decision is partly already provided. The "chained" half still holds and matches the upstream instruction: "Call the initialization functions after installing any other panic hooks". A hand-written `Drop` guard is worth keeping only if it does something `restore` does not. This is a note for `SPEC.md`, not a contradiction.

**Decision 9 (resize never changes the maze).** The design is supported and is simpler than the map implies. The app needs no `Event::Resize` arm. `Terminal::draw` calls `autoresize` for the fullscreen viewport, and `Frame::area()` is the documented source of truth.

**Decision 11 (one speed ladder against a ~60 Hz render tick).** The official poll-plus-tick loop does not guarantee a fixed tick count per second, because an input event ends the `poll` wait early. Drive the fractional step budget from measured elapsed time.

**Decision 20 (`TestBackend` snapshots).** Use `assert_buffer_lines`. `assert_buffer_eq!` is deprecated and unreachable through `ratatui`. Test helpers must not be typed `io::Result<()>`, because `TestBackend::Error` is `Infallible`.

**Decision 2 (CI matrix on Linux, Windows and macOS).** Set the toolchain to 1.88.0 or later. That is the ratatui MSRV.

**Decision 4 (no threads).** The crossterm docs require `poll` and `read` to run on one thread. The decision is compatible.

## Open points

- The exact release in which `assert_buffer_eq!` left the `ratatui` facade is not identified. `BREAKING-CHANGES.md` has no entry for it. The present state is verified.
- crossterm's own API docs do not state that Windows delivers a key press and a key release for one keystroke. The only primary statement found is the ratatui FAQ, cited above.
- No prose recipe for tick rate exists on the ratatui website. The guidance above comes from two official example programs.
