# How MazeLab draws: two columns, 16 colours, and glyphs that carry the ranking

Three choices that hold together. Any one of them read alone looks arbitrary.

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

## Glyph carries the ranking, not colour

Because there is no contrast guarantee, colour alone cannot express the cell
priority order. A monotone shade ramp carries it instead: `██`, `▓▓`, `▒▒`,
`░░`, blank. Start, goal and current cell are letters, `SS`, `GG` and `@@`, in
both the Unicode and the ASCII glyph sets. Every cell state is distinct in glyph
as well as in colour.

## `--ascii` is load-bearing

It is not a nicety for old terminals. Windows Terminal is the documented default
host from Windows 11 22H2 and its default face carries `U+2588` at one cell
width, but `conhost` remains reachable by group policy, and Microsoft documents
that a raster font cannot show Unicode at all.

## Open

How `--ascii` is reached is not settled here: a flag the user sets, or a runtime
capability probe that degrades on its own. That question, and the wider one of
how the cross-platform rendering claim is substantiated at all, is
[issue #11](https://github.com/daniel-kindl/mazelab/issues/11). Nothing above
depends on its answer.
