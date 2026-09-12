# Cross-platform terminal capabilities and CI matrix

Research for issue [#4](https://github.com/daniel-kindl/mazelab/issues/4). Part of map issue
[#1](https://github.com/daniel-kindl/mazelab/issues/1).

Date of research: 2026-09-12.

## How to read this file

Each claim carries one of three labels.

- **GUARANTEE** — a document states the behaviour. The document is the source of the claim.
- **SOURCE** — the code or the repository data shows the behaviour. No document states it. Treat it
  as an implementation detail. It can change without a breaking-change note.
- **UNVERIFIED** — no primary source states it. Do not rely on it.

Versions pinned for this research:

| Crate / image set | Version | Primary source |
| --- | --- | --- |
| `crossterm` | 0.29.0 (2025-04-05) | `https://crates.io/api/v1/crates/crossterm` |
| `ratatui` | 0.30.2 (2026-06-19) | `https://crates.io/api/v1/crates/ratatui` |
| `ratatui` MSRV | 1.88.0 | `ratatui` 0.30.2 `Cargo.toml`, `rust-version = "1.88.0"` |
| `actions/runner-images` | `main` at `bac2275`, 2026-09-11 | `https://github.com/actions/runner-images/blob/main/README.md` |

`ratatui` 0.30.2 selects `crossterm` 0.29.x by default. The `ratatui-crossterm` crate declares
`crossterm_0_29` as a default feature.

---

## 1. Does `U+2588 FULL BLOCK` show correctly on Windows?

### Windows Terminal

**GUARANTEE.** The default font face is Cascadia Mono. The documented fallback face is Consolas.

> Microsoft Learn, Windows Terminal profile appearance: "The terminal will try to fallback to
> Consolas if this can't be found or is invalid. ... **Default value:** `"Cascadia Mono"`"
> `https://learn.microsoft.com/en-us/windows/terminal/customize-settings/profile-appearance`

Note the exact scope of that fallback. It applies when the face is missing or invalid. It does not
apply when a valid face lacks one glyph.

**SOURCE.** Cascadia contains `U+2588`, and the glyph is one cell wide.

> `microsoft/cascadia-code`, `sources/CascadiaCode-Regular.ufo/glyphs/fullB_lock.glif`:
> `<glyph name="fullBlock" format="2"> <advance width="1200"/> <unicode hex="2588"/>`

The letter `a` in the same font source also has `<advance width="1200"/>`. So `U+2588` has the
advance of a normal Latin letter. The font source also contains `U+2580`, `U+2591`, and about 58
block and quadrant glyph names, which is consistent with full coverage of U+2580–U+259F.

**Not a guarantee.** Microsoft publishes no prose statement of Unicode coverage for Cascadia. The
only coverage claim in the `cascadia-code` README is an image caption: "Symbols for Legacy Computing
and other block elements". The caption names no Unicode range.

### The legacy console host, `conhost.exe`

**GUARANTEE.** A raster font cannot show Unicode characters. A TrueType font is required, and the
font must define the glyph.

> Microsoft Learn, `System.Console`: "To successfully display Unicode characters to the console, the
> console font must be set to a non-raster or TrueType font such as Consolas or Lucida Console."
> `https://learn.microsoft.com/en-us/dotnet/api/system.console`

> Microsoft Learn, `Console.OutputEncoding`: "A font used by the console must define the particular
> glyph or glyphs to be displayed. The console can take advantage of font linking to display glyphs
> from linked fonts if the base font does not contain a definition for that glyph."
> `https://learn.microsoft.com/en-us/dotnet/api/system.console.outputencoding`

Read "can take advantage of" as permissive wording. It is not a promise.

**UNVERIFIED.** The default font face of `conhost`. Microsoft documents a codepage-dependent default
(`FaceName` = `__DefaultTTFont__` in `microsoft/terminal`, `doc/ConsoleHostSettings.md`), not a fixed
face.

**UNVERIFIED.** Whether Consolas or Lucida Console cover U+2580–U+259F. Microsoft publishes no glyph
list for either font.

### Which consoles are a risk

| Host | Risk | Basis |
| --- | --- | --- |
| Windows Terminal, default profile | Low | Cascadia Mono is the documented default; the font source contains `U+2588` at one-cell width |
| `conhost.exe` with a raster font | **High** | Microsoft documents that a raster font cannot show Unicode |
| `conhost.exe` with Consolas or Lucida Console | Conditional, unverified | TrueType is necessary but not sufficient; glyph coverage is not published |
| Any host with a user-chosen face that lacks block glyphs | Conditional, unverified | Face-level fallback does not trigger on a missing glyph; the width of a font-linked glyph in the console grid is not documented |

### Which console is the default now

**GUARANTEE.** Windows Terminal is the default console host from Windows 11 22H2.

> Microsoft Support: "In Windows 11 22H2, the default app used to host console windows has been
> changed to Windows Terminal."
> `https://support.microsoft.com/en-us/windows/command-prompt-and-windows-powershell-for-windows-11-6453ce98-da91-476f-8651-5c14d5777c20`

> Microsoft Learn, Windows Terminal group policy: "Supported on Windows 11 22H2 or Windows 10 22H2
> (Build 19045.3031, KB5026435) with Windows Terminal 1.17 or later, this policy lets you select the
> default terminal application used in Windows."
> `https://learn.microsoft.com/en-us/windows/terminal/group-policy`

The group policy offers "Windows Console Host (legacy)" as a choice. A user can still select it.

**GUARANTEE.** `conhost` is not removed. Its user interface is retired from the default position; the
process stays as the console API server.

> Microsoft Learn, console ecosystem roadmap: "Inside the Windows operating system, this includes
> retiring the classic console host user interface from its default position in favor of Windows
> Terminal, ConPTY, and virtual terminal sequences."
> `https://learn.microsoft.com/en-us/windows/console/ecosystem-roadmap`

> Microsoft Learn, classic vs VT: "There are no plans to remove the Windows console APIs from the
> platform." `https://learn.microsoft.com/en-us/windows/console/classic-vs-vt`

One Microsoft Learn page contradicts the above. The Windows Terminal startup-settings page still
states "**Default value:** Windows Console Host". Its `ms.date` is 2022-02-03. The group-policy and
install pages carry `ms.date` 2025-11-10 and take precedence.

### Cell width

**GUARANTEE.** A console cell is one cell or two cells wide, and a one-cell character is taller than
it is wide.

> Microsoft Learn, console ecosystem roadmap: "expanding representation of a cell in the console
> environment to be either 1 or 2 cells wide, where 1 cell is narrow (taller than it is wide) and 2
> cells is wide, full-width".

This is the documented reason that map decision 15 needs two screen columns per display-grid
position. The decision is sound.

**UNVERIFIED.** Microsoft states no East Asian width class for `U+2588`, and states nothing about how
either host resolves an ambiguous-width character. The one-cell advance in the Cascadia font source is
SOURCE evidence, not a guarantee.

### Verdict for point 1

Block-glyph rendering is safe on Windows Terminal with its default font, on the strength of the font
source. It is **not safe** on `conhost` with a raster font, and coverage is **unverified** on
`conhost` with a TrueType font. `conhost` remains selectable by group policy. The `--ascii` flag of
map decision 15 is therefore load-bearing, not optional.

---

## 2. Does `crossterm` need an explicit call to enable virtual terminal processing?

### What Windows requires

**GUARANTEE.** An application must opt in. The flag is not among the modes enabled when a screen
buffer is created.

> Microsoft Learn, console virtual terminal sequences: "The following terminal sequences are
> intercepted by the console host when written into the output stream, if the
> ENABLE_VIRTUAL_TERMINAL_PROCESSING flag is set on the screen buffer handle using the SetConsoleMode
> function."
> `https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences`

> Microsoft Learn, high-level console modes: "Both of the following console output modes are enabled
> for a console screen buffer when it is created: Processed output mode; Wrapping at EOL output mode"
> `https://learn.microsoft.com/en-us/windows/console/high-level-console-modes`

`ENABLE_VIRTUAL_TERMINAL_PROCESSING` is absent from that list.

**UNVERIFIED.** Whether Windows Terminal changes the default console mode of the attached client. No
Microsoft source states that it does. The documented architecture puts `conhost` or ConPTY in the
server role, and the server owns the console mode that the process reads.

### What `crossterm` documents

**There is no documented guarantee.** No public `crossterm` 0.29.0 document states that any call
enables `ENABLE_VIRTUAL_TERMINAL_PROCESSING`. The nearest statements describe the fallback, not the
opt-in:

> `crossterm` crate root: "Linux and Windows 10 systems support ANSI escape codes. ... For older
> windows systems a WinAPI call is made." `https://docs.rs/crossterm/0.29.0/crossterm/`

### What the source does

**SOURCE.** `crossterm` enables the flag once per process, as a side effect of the command-dispatch
path.

`src/command.rs`, `QueueableCommand::queue`, calls `command.is_ansi_code_supported()` on Windows. That
method calls `ansi_support::supports_ansi()`. `supports_ansi()` runs a `parking_lot::Once` that calls
the private `enable_vt_processing()`, which calls `SetConsoleMode` on
`Handle::current_out_handle()`.

So the first `crossterm` command that is queued or executed on Windows is what enables VT processing.
For a `ratatui` application that is normally `EnterAlternateScreen`.

Four further SOURCE facts that matter:

- `enable_raw_mode()` does **not** enable VT processing. `src/terminal/sys/windows.rs` acts on the
  console **input** handle and clears `ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT`.
  It never references `ansi_support`. `enable_raw_mode` is a plain function, not a `Command`, so it
  never reaches the dispatch path.
- `supports_ansi()` returning `true` does **not** imply that VT processing was enabled. The function
  also returns `true` when `enable_vt_processing()` fails but `TERM` is set to anything other than
  `dumb`. That branch exists for Git Bash.
- `crossterm` never clears the flag again. There is no restore at teardown.
- `crossterm` never sets `ENABLE_VIRTUAL_TERMINAL_INPUT`. Windows raw mode gives console input
  records, not VT input sequences.

### Public API

`crossterm::ansi_support::supports_ansi()` is public, but `#[cfg(windows)]`. `Cargo.toml` sets
`all-features = true` for docs.rs without a `targets` key, so docs.rs builds the Linux target only.
Both `https://docs.rs/crossterm/0.29.0/crossterm/ansi_support/index.html` and the function page
return HTTP 404. The function is effectively undocumented.

### Verdict for point 2

MazeLab does not need its own `SetConsoleMode` call in practice. It relies on an undocumented
side effect to get one. Record this as a known risk. If a future `crossterm` release moves the
opt-in, the symptom is raw escape sequences printed on Windows. The `--ascii` flag does not help
against that symptom, because the problem is the escape sequences, not the glyphs.

Do not put `enable_raw_mode()` alone before the first draw and assume VT is on.

---

## 3. How do the 16 ANSI named colours map, and is any one unreliable?

### Windows

**GUARANTEE.** All 32 SGR colour codes are documented as supported: 30–37, 40–47, 90–97, 100–107, plus
`0`, `1`, `22`, `39`, `49`, `7`, `27`.

**GUARANTEE.** The named colours are palette indices. Their RGB comes from the console colour table,
which the user controls.

> Microsoft Learn, console virtual terminal sequences: "For options that specify colors, the colors
> will be used as defined in the console color table which can be modified using the
> SetConsoleScreenBufferInfoEx API. If the table is modified to make the "blue" position in the table
> display an RGB shade of red, then all calls to Foreground Blue will display that red color until
> otherwise changed."

This is the documented basis for map decision 16. The decision is correct: the 16 named colours do
inherit the user's terminal theme, on Windows as well.

**GUARANTEE.** Truecolour degrades on the Windows Console.

> Same page: "For these extended colors, the Windows Console will choose the nearest appropriate color
> from the existing 16 color table for display."

This supports the map's placement of the truecolor expansion gradient in v2, out of scope for v1.

**GUARANTEE.** Windows Terminal's default colour scheme is Campbell, and `\x1b[1m` is rendered bright
by default.

> Microsoft Learn, profile appearance: "**Default value:** `"Campbell"`" and, for intense text
> formatting, "`"bright"`: render intense text bright, but not bold ... **Default value:** `"bright"`"

The consequence is a caveat for MazeLab. `\x1b[1m` with SGR 30–37 is not a reliable way to reach a
bright colour, because the user can set `intenseTextStyle` to `"bold"` or `"none"`. Use SGR 90–97 —
that is, use `crossterm`'s light variants — where a bright colour is wanted.

**GUARANTEE, with an unverified default.** Windows Terminal can change the foreground colour by
itself.

> Same page, "Adjust indistinguishable colors": "This setting adjusts the foreground color to make it
> more visible, based on the background color. ... **Accepts:** `always`, `indexed`, `never`"

The page states no default value. UNVERIFIED which value ships.

**GUARANTEE.** Do not mix the two colour APIs.

> Microsoft Learn, Windows Terminal troubleshooting: "Choose either Windows API functions or VT
> sequences for adjusting colors. Don't mix them."
> `https://learn.microsoft.com/en-us/windows/terminal/troubleshooting`

**No Microsoft source singles out a named colour as unreliable.** What is documented is the general
rule: every named colour is a palette index with no RGB guarantee.

### `crossterm`

**GUARANTEE.** `crossterm` documents 16 named variants in eight light/dark pairs, and states they work
on almost all terminals.

> `crossterm::style::Color`: "The following list of 16 base colors are available for almost all
> terminals (Windows 7 and 8 included)."
> `https://docs.rs/crossterm/0.29.0/crossterm/style/enum.Color.html`

The documented pairing is: `DarkGrey`/`Black`, `Red`/`DarkRed`, `Green`/`DarkGreen`,
`Yellow`/`DarkYellow`, `Blue`/`DarkBlue`, `Magenta`/`DarkMagenta`, `Cyan`/`DarkCyan`, `White`/`Grey`.
The left column is the light member. Note that `White` is the light member and `Grey` is the dark
member, and that `DarkGrey` is the light member of the `Black` pair. The names do not read the way the
pairing works.

**GUARANTEE.** `Color::Rgb` and `Color::AnsiValue` are "Most UNIX terminals and Windows 10 supported
only."

**SOURCE, not documented.** On the WinAPI path, `Color::Rgb` and `Color::AnsiValue` both map to `0`,
which is black. `src/style/sys/windows.rs`:

```rust
Color::Rgb { .. } => 0,
Color::AnsiValue(_val) => 0,
```

The documentation says these are Windows-10-only. It does not say the fallback is a silent remap to
black. MazeLab uses named colours only (map decision 16), so this does not apply, but it is a reason
not to relax decision 16 casually.

### The one named colour with a documented problem

**`DarkYellow` on PowerShell, on the WinAPI path.** `crossterm` ships `docs/know-problems.md` inside
the crate tarball. It is not rendered on docs.rs and is not linked from the README.

> "Power shell does not interpreter 'DarkYellow' and is instead using gray instead, cmd is working
> perfectly fine." `https://github.com/crossterm-rs/crossterm/blob/0.29/docs/know-problems.md`

The same file records that PowerShell resets the background and foreground colours after the program
runs.

A historical `CHANGELOG.md` entry records "Windows 7 grey/white foreground/intensity swapped". That is
fixed and applies to Windows 7 only.

### An open bug in 0.29.0

`crossterm` 0.29.0 emits a bare `CSI m` when colours are disabled through `NO_COLOR`. `CSI m` resets
every attribute instead of doing nothing. The fix is on `master` and is unreleased. This affects
`SetForegroundColor`, `SetBackgroundColor`, `SetUnderlineColor`, and `SetColors`.

### Verdict for point 3

`DarkYellow` is the one named colour with a documented problem, and the problem is confined to
PowerShell on the WinAPI path. Avoid `DarkYellow` as a load-bearing colour in the priority order of
map decision 17, or accept that it can read as grey.

A second, larger caveat falls directly out of map decision 16. Because every named colour is a
theme-controlled palette index, there is **no documented contrast guarantee between any two named
colours**. Under a user scheme, two cell states that decision 17 ranks differently can render at the
same or a near-identical RGB. Black and dark grey, and grey and white, are the obvious pairs. Decision
16 buys theme integration and pays for it with the loss of any contrast promise. This is a trade the
map should record, not a defect.

---

## 4. Raw mode and the alternate screen: Windows versus Unix

### What raw mode documents

**GUARANTEE, and it is platform-independent.** `crossterm` documents one list for both platforms.

> `crossterm::terminal`, "Raw Mode": "Those modes will be set when enabling raw modes: Input will not
> be forwarded to screen; Input will not be processed on enter press; Input will not be line buffered
> (input sent byte-by-byte to input buffer); Special keys like backspace and CTRL+C will not be
> processed by terminal driver; New line character will not be processed therefore `println!` can't be
> used, use `write!` instead."
> `https://docs.rs/crossterm/0.29.0/crossterm/terminal/index.html#raw-mode`

**No Windows/Unix difference in what raw mode disables is documented.**

### The difference that exists in the source

**SOURCE.** The two platforms differ in how `disable_raw_mode` works.

- Unix, `src/terminal/sys/unix.rs`: `enable_raw_mode` snapshots the whole termios into a global, then
  calls `make_raw()`. It is idempotent — it returns early when a prior mode is already stored.
  `disable_raw_mode` restores the **entire saved termios**.
- Windows, `src/terminal/sys/windows.rs`: `enable_raw_mode` clears three bits on the console input
  handle. It saves nothing. `disable_raw_mode` ORs those three bits back on.

So on Windows, `disable_raw_mode()` **sets** `ENABLE_LINE_INPUT`, `ENABLE_ECHO_INPUT` and
`ENABLE_PROCESSED_INPUT` whether or not they were set before MazeLab started. It is not a restore.

`is_raw_mode_enabled()` also differs. On Unix it reports whether `crossterm` itself enabled raw mode.
On Windows it queries the live console mode bits.

### The alternate screen on Windows

**No condition is documented.** The `EnterAlternateScreen` and `LeaveAlternateScreen` docs name no
platform and no console host. Their full "Notes" sections say only that commands must be executed or
queued, and point at each other.

`crossterm`'s README lists "Alternate screen" and "Raw screen" as features, and its "Tested Terminals"
table lists `Console Host` on Windows 10 and 8.1, and `Windows Terminal` on Windows 10 x86_64 and
Windows 11 arm64. A tested-terminals table is evidence of testing, not a support guarantee.

**SOURCE.** The WinAPI fallback creates a real second console screen buffer
(`ScreenBuffer::create()?` then `.show()?`), so the alternate screen works on legacy `conhost` without
VT processing.

**GUARANTEE, and a scope limit.** `crossterm` supports exactly two screens.

> `crossterm::terminal`: "crossterm does not yet support creating screens, or switch between more than
> two buffers, and only offers the ability to change between the 'alternate' and 'main screen'."

### The teardown order

This is the known difference the ticket asks for, and the two crates disagree.

**`crossterm` documents no ordering requirement.** Nothing in the 0.29.0 rustdoc, README or CHANGELOG
constrains the order. Its one example that uses both, `examples/stderr.rs`, does
`LeaveAlternateScreen` **first**, then `disable_raw_mode()`.

**`ratatui` documents the opposite order, as a numbered procedure.**

> `ratatui::restore` and `ratatui::try_restore`: "This function will attempt to restore the terminal
> to its original state by performing the following steps: 1. Raw mode is disabled. 2. The alternate
> screen buffer is left." `https://docs.rs/ratatui/0.30.2/ratatui/fn.try_restore.html`

The `ratatui` source carries the rationale as a comment, not as a specification:

```rust
// disabling raw mode first is important as it has more side effects than leaving the alternate
// screen buffer
disable_raw_mode()?;
execute!(stdout(), LeaveAlternateScreen)?;
```

`ratatui` is internally inconsistent. The `CrosstermBackend` struct docs show the other order in their
example. So the documented ordering lives on `restore`/`try_restore` alone.

**Use the `ratatui::try_restore` order: disable raw mode first, then leave the alternate screen.** It
is the only ordering any primary document states as a procedure.

### Interaction with map decision 18

Decision 18 wants a `Drop` guard plus a chained panic hook. `ratatui`'s own init functions already
install a panic hook, and `ratatui` documents an ordering constraint on it:

> `ratatui::init`: "All initialization functions install a panic hook that automatically restores the
> terminal state before panicking. ... **Important**: Call the initialization functions *after*
> installing any other panic hooks to ensure the terminal is restored before other hooks run."
> `https://docs.rs/ratatui/0.30.2/ratatui/init/index.html`

So decision 18 must either chain onto `ratatui`'s hook by installing MazeLab's hook first and then
calling `ratatui::init`, or skip `ratatui::init` and do setup and teardown directly. Decide this in
`SPEC.md`.

**GUARANTEE.** `CrosstermBackend` does no terminal setup by itself.

> `ratatui_crossterm::CrosstermBackend`: "This is not done automatically by the backend because it is
> possible that the application may want to use the terminal for other purposes (like showing help
> text) before entering alternate screen mode."

### Verdict for point 4

Raw mode and the alternate screen both work on Windows and on Unix, and `crossterm` documents one
behaviour for both. The documented difference in teardown is the ordering: `ratatui` specifies raw
mode off first, then leave the alternate screen. Follow it. The undocumented difference is that
Windows `disable_raw_mode` sets three input bits rather than restoring what was there before.

---

## 5. GitHub Actions runner labels and a minimal matrix

Read from `actions/runner-images` `main` at commit `bac2275`, dated 2026-09-11. Read on 2026-09-12.
Cross-checked against
`https://docs.github.com/en/actions/reference/runners/github-hosted-runners`, which agrees on all
three `-latest` mappings.

### Current labels

| Image | Arch | Labels | Status |
| --- | --- | --- | --- |
| Ubuntu 26.04 | x64 | `ubuntu-26.04` | preview |
| Ubuntu 26.04 Arm64 | arm64 | `ubuntu-26.04-arm` | preview |
| Ubuntu 24.04 | x64 | `ubuntu-latest`, `ubuntu-24.04` | available |
| Ubuntu 24.04 Arm64 | arm64 | `ubuntu-24.04-arm` | available |
| Ubuntu 22.04 | x64 | `ubuntu-22.04` | available, deprecating |
| Ubuntu 22.04 Arm64 | arm64 | `ubuntu-22.04-arm` | available, deprecating |
| Ubuntu Slim | x64 | `ubuntu-slim` | available |
| Windows Server 2025 | x64 | `windows-latest`, `windows-2025`, `windows-2025-vs2026` | available |
| Windows Server 2022 | x64 | `windows-2022` | available |
| Windows 11 Arm64 | arm64 | `windows-11-arm` | available |
| Windows 11 Arm64, VS 2026 | arm64 | `windows-11-vs2026-arm` | available |
| macOS 26 | x64 | `macos-latest-large`, `macos-26-intel`, `macos-26-large` | available |
| macOS 26 Arm64 | arm64 | `macos-latest`, `macos-26`, `macos-26-xlarge` | available |
| macOS 15 | x64 | `macos-15-large`, `macos-15-intel` | available |
| macOS 15 Arm64 | arm64 | `macos-15`, `macos-15-xlarge` | available |
| macOS 14 | x64 | `macos-14-large` | **deprecated** |
| macOS 14 Arm64 | arm64 | `macos-14`, `macos-14-xlarge` | **deprecated** |
| Xcode 27 | arm64 | `xcode-27`, `xcode-27-xlarge` | preview |

The `-latest` pointers today:

- `ubuntu-latest` → Ubuntu 24.04, x64. **Not** 26.04. No migration announcement exists yet.
- `windows-latest` → Windows Server 2025, x64, and that image now carries Visual Studio 2026. The
  README places `windows-latest`, `windows-2025` and `windows-2025-vs2026` on one row.
- `macos-latest` → macOS 26, **arm64 (Apple Silicon)**. `macos-latest-large` is the Intel image.

`windows-2019` is absent from the README. It is gone.

### Deprecations with dates

- **macOS 14.** Deprecation began 2026-07-06. Images are fully unsupported from **2026-11-02**.
  Brownouts that fail jobs run on Oct 5–6, 12–13, 16–17, 19–20, 23–24, 26–27, 29–30 and 30–31, each
  from 14:00 UTC to 00:00 UTC. Affects `macos-14`, `macos-14-large`, `macos-14-xlarge`.
  (`actions/runner-images` issue #13518.)
- **Ubuntu 22.04.** Deprecation begins **2026-09-17**, five days after this research. Images are fully
  unsupported from 2027-04-17. Affects `ubuntu-22.04` and `ubuntu-22.04-arm`.
  (`actions/runner-images` issue #14254.)
- **`windows-11-arm`.** The label moves to the Visual Studio 2026 image over the week beginning
  2026-09-21, completing by 2026-09-30. (`actions/runner-images` issue #14602.)
- **`windows-2022`.** No open deprecation announcement. Not verifiable as deprecated.

A beta image carries no SLA:

> `actions/runner-images` README: "Any workflows that run on a beta image do not fall under the
> customer SLA."

### Rust on the images

Every Ubuntu, Windows and macOS image documents Rust 1.98.x and `rustup`. `ubuntu-slim` has no Rust
section at all. `ratatui`'s MSRV is 1.88.0, so no toolchain action is needed.

`clippy` is where the documentation is uneven.

| Image readme | `clippy` listed? |
| --- | --- |
| `Ubuntu2404-Readme.md`, `Ubuntu2404-Arm64-Readme.md`, `Ubuntu2604-Readme.md` | **No** |
| `Ubuntu2204-Readme.md` | Yes, Cargo clippy 0.1.98 |
| `Windows2025-VS2026-Readme.md` (= `windows-latest`), `Windows2022-Readme.md`, `Windows11-Arm64-Readme.md` | Yes, Clippy 0.1.98 |
| `macos-26-arm64-Readme.md` (= `macos-latest`), `macos-26-Readme.md`, `macos-15-arm64-Readme.md` | Yes, Clippy 0.1.98 |

The omission is a documentation-generation artifact, not a missing tool.
`images/ubuntu/scripts/build/install-rust.sh` runs `rustup component add rustfmt clippy`
unconditionally, but `images/ubuntu/scripts/docs-gen/Generate-SoftwareReport.ps1` only emits the
Clippy line when `Test-IsUbuntu22` is true.

The documented guarantee is therefore weaker than the actual state: for `ubuntu-latest`, no readme
states that `clippy` is installed. The workflow below runs `rustup component add clippy` to remove the
dependency on an undocumented fact. The command is idempotent and cheap.

### Proposed workflow

Place it at `.github/workflows/ci.yml`.

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  check:
    name: ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-24.04, windows-2025, macos-26]

    steps:
      - uses: actions/checkout@v7

      - name: Show toolchain
        run: |
          rustc --version
          cargo --version

      - name: Add clippy
        run: rustup component add clippy

      - name: Build
        run: cargo build --locked --all-targets

      - name: Test
        run: cargo test --locked

      - name: Clippy
        run: cargo clippy --locked --all-targets -- -D warnings
```

`actions/checkout` v7 is the current major version. The latest release is v7.0.1, dated 2026-07-20.

Choices, and the reason for each:

- **Explicit labels, not `-latest`.** `windows-latest` moved to Visual Studio 2026 in June 2026 and
  `macos-latest` moved to macOS 26 in the same month. An explicit label makes the image a reviewed
  change instead of a silent one. The cost is that the labels must be bumped by hand. Ubuntu 24.04,
  Windows Server 2025 and macOS 26 are exactly what the three `-latest` labels point at today, so the
  matrix starts on the same images either way.
- **`macos-26` is arm64.** So is `macos-latest`. The matrix covers Apple Silicon only. Intel macOS
  needs `macos-26-intel` as a fourth entry. MazeLab has no architecture-specific code, so one macOS
  architecture is defensible for v1. Record the choice.
- **`fail-fast: false`.** A platform-specific failure is the failure this matrix exists to find.
  Stopping the other two legs hides it.
- **`--locked`.** The build must use the committed `Cargo.lock`, or the matrix is not reproducing what
  a developer builds.
- **`-D warnings` on clippy.** Without it, `cargo clippy` exits `0` on a lint and the step proves
  nothing.
- **`cargo test` without `--all-targets`.** `--all-targets` skips doctests. The build step already
  covers `--all-targets` compilation.
- No caching. Add `Swatinem/rust-cache` later if the matrix becomes slow. Correctness first.

If the matrix later needs the widest cheap coverage, add `ubuntu-24.04-arm` and `windows-11-arm`. Do
not add `ubuntu-22.04` (deprecating from 2026-09-17) or any `macos-14` label (unsupported from
2026-11-02).

---

## 6. Can a `ratatui` `TestBackend` test run on all three runners?

**Yes.** `TestBackend` needs no terminal, no TTY, and no OS terminal handle.

**GUARANTEE.** The documented purpose is an in-memory buffer.

> `ratatui_core::backend::TestBackend`: "A `Backend` implementation used for integration testing that
> renders to an memory buffer." (The typo is in the source and in the rendered docs.)
> `https://docs.rs/ratatui-core/0.1.2/ratatui_core/backend/struct.TestBackend.html`

**GUARANTEE.** The backend cannot fail with an I/O error. The error type is `Infallible`.

> `ratatui` `BREAKING-CHANGES.md`, v0.30.0: "`TestBackend` now uses `core::convert::Infallible` for
> error handling instead of `std::io::Error`"

An `Infallible` error type is the strongest available evidence that no syscall is involved.

**SOURCE.** `ratatui-core/src/backend/test.rs` holds only buffers:

```rust
pub struct TestBackend {
    buffer: Buffer,
    scrollback: Buffer,
    cursor: bool,
    pos: (u16, u16),
}
```

`draw()` writes into that buffer. `size()` returns `Ok(self.buffer.area.as_size())` — no ioctl and no
terminal query. The file imports `alloc` and `core` only. It contains no match for `std::`,
`crossterm`, `termion`, `libc` or `windows`. `ratatui-core` is `#![no_std]`. The only `cfg` attributes
in the file are `feature = "scrolling-regions"` and `test`.

**No feature flag gates it.** In `ratatui/src/lib.rs`, `TestBackend` is re-exported ungated, unlike
the neighbouring `CrosstermBackend`, which carries `#[cfg(feature = "crossterm")]`. docs.rs shows no
"Available on crate feature" badge on the `TestBackend` page.

**GUARANTEE.** `Terminal::new(TestBackend::new(w, h))` is the construction the `ratatui` doc example
itself uses:

```rust
/// # use ratatui_core::{backend::TestBackend, terminal::Terminal};
/// # let backend = TestBackend::new(10, 10);
/// # let _terminal = Terminal::new(backend)?;
```

`Terminal::new` calls only `size()` on the backend during construction, which `TestBackend` answers
from its own buffer.

The default features of `ratatui` 0.30.2 are `["all-widgets", "crossterm", "layout-cache", "macros",
"underline-color"]`. `crossterm` is therefore compiled in by default, but `TestBackend` does not use
it. MazeLab needs `crossterm` for the real backend anyway, so leave the defaults alone.

### Verdict for point 6

`TestBackend` tests run on `ubuntu-24.04`, `windows-2025` and `macos-26` with no TTY. This matches map
decision 20, which states that nothing drives the real event loop.

---

## What this research means for the map

### Decision 15 — block glyphs at two columns, with `--ascii`

**Supported, with one correction of emphasis.** The two-columns-per-position choice has a documented
basis: Microsoft states that a one-cell console character is "taller than it is wide". The `--ascii`
flag is not a nicety. It is the only mitigation for `conhost` with a raster font, which Microsoft
documents as unable to show Unicode at all. Also note that Cascadia's block coverage is SOURCE
evidence from the font repository, not a documented guarantee.

### Decision 16 — 16 ANSI named colours only

**Supported, with a consequence that should be written down.** Microsoft documents explicitly that the
named colours are indices into a user-modifiable colour table, which is exactly the theme inheritance
the decision wants. The consequence is that **no contrast guarantee exists between any two named
colours**. Under some user scheme, two cell states that decision 17 ranks differently can render
identically. Decision 17 should not rely on colour alone to separate two adjacent priority levels
where the distinction carries meaning.

One named colour carries a documented defect: `DarkYellow` renders as grey in PowerShell on the WinAPI
path.

### Decision 2 — CI matrix as the only evidence

**This is the finding that contradicts the map.** Decision 2 says the CI matrix is the only evidence
that the cross-platform claim holds. The matrix cannot carry that weight for the rendering half of the
claim.

The matrix proves that MazeLab compiles, that its tests pass, and that it is lint-clean on three
platforms. `TestBackend` renders into memory. It never touches a terminal, a font, a colour table or
`SetConsoleMode`. So the matrix supplies **no evidence at all** about:

- whether `U+2588` has a glyph in the user's font;
- whether two block characters occupy exactly two columns;
- whether `ENABLE_VIRTUAL_TERMINAL_PROCESSING` was enabled;
- whether any two named colours are distinguishable under the user's scheme.

Decision 2 should be restated. The CI matrix is evidence of **portable compilation and portable
logic**. The rendering claim rests on the documented guarantees in this file plus the `--ascii`
escape hatch, and it is not verified by any automated test. Since decision 2 also rules out manual
verification on real hardware, the honest position is that the rendering claim is argued from
documentation, not tested.

### Decision 18 — `Drop` guard plus chained panic hook

**Two constraints arrive from `ratatui` that `SPEC.md` must resolve.** Teardown must disable raw mode
first and leave the alternate screen second, per `ratatui::try_restore`. And `ratatui`'s init
functions install their own panic hook, with a documented requirement that any other panic hook be
installed **before** `ratatui::init` is called.

### New risk to record

MazeLab depends on an undocumented `crossterm` side effect to enable virtual terminal processing on
Windows. See point 2. There is no documented guarantee, and no CI test can detect its loss.

---

## Sources

- `https://docs.rs/crossterm/0.29.0/crossterm/`
- `https://github.com/crossterm-rs/crossterm/blob/0.29/` — `src/`, `README.md`, `CHANGELOG.md`,
  `docs/know-problems.md`, `examples/stderr.rs`
- `https://docs.rs/ratatui/0.30.2/ratatui/`, `https://docs.rs/ratatui-core/0.1.2/ratatui_core/`,
  `https://docs.rs/ratatui-crossterm/0.1.2/ratatui_crossterm/`
- `https://github.com/ratatui/ratatui/blob/ratatui-v0.30.2/` — `ratatui/src/init.rs`,
  `BREAKING-CHANGES.md`, `Cargo.toml`
- `https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences`
- `https://learn.microsoft.com/en-us/windows/console/setconsolemode`
- `https://learn.microsoft.com/en-us/windows/console/high-level-console-modes`
- `https://learn.microsoft.com/en-us/windows/console/classic-vs-vt`
- `https://learn.microsoft.com/en-us/windows/console/ecosystem-roadmap`
- `https://learn.microsoft.com/en-us/windows/console/char-info-str`
- `https://learn.microsoft.com/en-us/windows/terminal/customize-settings/profile-appearance`
- `https://learn.microsoft.com/en-us/windows/terminal/customize-settings/color-schemes`
- `https://learn.microsoft.com/en-us/windows/terminal/group-policy`
- `https://learn.microsoft.com/en-us/windows/terminal/install`
- `https://learn.microsoft.com/en-us/windows/terminal/troubleshooting`
- `https://learn.microsoft.com/en-us/dotnet/api/system.console`
- `https://learn.microsoft.com/en-us/dotnet/api/system.console.outputencoding`
- `https://support.microsoft.com/en-us/windows/command-prompt-and-windows-powershell-for-windows-11-6453ce98-da91-476f-8651-5c14d5777c20`
- `https://github.com/microsoft/terminal` — `README.md`, `doc/ConsoleHostSettings.md`
- `https://github.com/microsoft/cascadia-code` — `README.md`,
  `sources/CascadiaCode-Regular.ufo/glyphs/`
- `https://github.com/actions/runner-images/blob/main/README.md` and the `images/*/`-Readme files,
  issues #13518, #14017, #14167, #14226, #14254, #14404, #14602
- `https://docs.github.com/en/actions/reference/runners/github-hosted-runners`
- `https://crates.io/api/v1/crates/crossterm`, `https://crates.io/api/v1/crates/ratatui`
- `https://github.com/actions/checkout` releases
