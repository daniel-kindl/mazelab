# How MazeLab draws: two columns, 16 colours, and glyphs that carry the meaning

Four choices that hold together. Any one of them read alone looks arbitrary.

## Two screen columns per display-grid position

A console cell is documented as taller than it is wide, so one screen cell per
position would draw a maze of tall thin rectangles. Two columns per position
make a cell look square. The footprint is `4W+2` by `2H+1`.

Block glyphs are used rather than box-drawing lines. Box-drawing draws a wall as
a line and leaves nothing to shade: every display-grid position here has to
carry a cell state of its own, with its own glyph and its own colour.

## 16 ANSI named colours only

The palette uses the 16 named colours, so it inherits the user's terminal theme.
The price is real and is the reason this is recorded: **named colours carry no
contrast guarantee.** The user controls what each one renders as, so two cell
states that the priority order ranks apart can appear identical. `DarkYellow` is
a documented offender, and `Yellow` is therefore banished from the maze palette.

This is what stands between MazeLab and a truecolor gradient shaded by expansion
order, which is the most striking visual addition available later. Reversing it
means giving up theme inheritance and adding runtime capability detection.

## The glyph alone identifies the cell state

Because there is no contrast guarantee, colour cannot carry a distinction on its
own. The rule is therefore stronger than a ranking: **the glyph alone identifies
the cell state, and colour only reinforces it.**

A weaker rule was considered and rejected: that no two states *adjacent in the
priority order* may differ by colour alone. Adjacency is the wrong axis. A user
theme can collide any two named colours, not only two that the order puts side
by side, and a reader who looks at a cell identifies that cell. The reader does
not compare it with its neighbour in the order.

A monotone shade ramp carries the ranking: `██`, `▓▓`, `▒▒`, `░░`, blank. Start,
goal and current cell are letters, `SS`, `GG` and `@@`, in both glyph sets.

Three unit tests hold the rule, because a later edit can break it silently:

1. No two cell states in one phase share a glyph, in either glyph set.
2. Every glyph in the ASCII glyph set is below `U+0080`.
3. `DarkYellow` does not appear in the maze palette.

The consequence is a claim that the tests prove: MazeLab stays legible when
colour is absent or when the theme renders two of its colours alike.

## `--ascii` is a flag, not a probe

`--ascii` is not a nicety for old terminals. Windows Terminal is the documented
default host from Windows 11 22H2 and its default face carries `U+2588` at one
cell width, but `conhost` remains reachable by group policy, and Microsoft
documents that a raster font cannot show Unicode at all.

The user selects the glyph set. MazeLab does not detect it. No documented signal
answers the question that matters, which is whether the font can draw the glyph:

- The host is not the font. `WT_SESSION` identifies Windows Terminal, which
  renders with a user-chosen face that can lack block glyphs.
- The encoding is not the font. `GetConsoleOutputCP` and `LANG` report which
  characters may be emitted. Microsoft states that a TrueType font is necessary
  for Unicode and is not sufficient, and exposes no API for the sufficient half.
- **The width probe does not work, although it looks like it should.** Writing a
  glyph and then reading the cursor column with `CSI 6n` measures the advance
  width. A correct `█` and a replacement box both advance one cell, so the probe
  cannot tell them apart. On Windows it measures the wrong layer as well:
  crossterm's `position()` does not use `CSI 6n` there, it reads the console
  screen buffer, which reports the text-buffer rules and not the renderer.

Any such probe could only ever downgrade safely, and none of them detects the
documented risk. A flag the user sets is the honest mechanism.

## The legend row is the evidence

No test can render a glyph on a real font, and no test can measure the contrast
of a user's theme. The legend row is what closes that gap, so it is part of this
decision and not layout furniture: **it shows every cell state of the current
phase, in the live glyph set and the live colour, and it never drops an entry.**

A user on any terminal can therefore see, without running a diagnostic, that a
glyph is missing or that two states render alike. A dedicated check screen was
considered and rejected: it duplicates the legend, and it is reached only by a
user who already suspects the problem that the legend has been showing.

The legend fits one row at every supported width. Labels shorten below 100
columns, which is the breakpoint the help row already uses. `frontier` and
`expanded` are glossary terms and are never shortened. The swatch for a state
whose glyph is blank is bracketed, because a blank swatch demonstrates nothing.

## Before you bump crossterm

Windows needs `ENABLE_VIRTUAL_TERMINAL_PROCESSING`, and crossterm 0.29 enables
it as an **undocumented side effect** of the first queued command. No crossterm
document promises this. If a release moves the opt-in, Windows users see raw
escape sequences, `--ascii` does not help, and no CI job can detect it.

MazeLab accepts this rather than calling `SetConsoleMode` itself. The committed
`Cargo.lock` is what bounds the risk: the behaviour can only change when someone
bumps crossterm on purpose. That person is the reader of this section. Check
that the first queued command still enables VT processing, on Windows, by hand.
