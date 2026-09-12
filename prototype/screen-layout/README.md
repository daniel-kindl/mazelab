# Screen mock (throwaway)

A static `ratatui` mock of the MazeLab screen. It answers
[issue #6](https://github.com/daniel-kindl/mazelab/issues/6): what does the
screen look like, and what are the keys?

**This is not the start of the application.** It has no event loop in the
MazeLab sense, no tick, no step budget, and no algorithm. Every number and
every cell on screen is a literal in `src/mock.rs`.

## Run it

```
cargo run
```

Give it a terminal of at least 100 x 26 to see the layouts at full size.

| key | what it does |
| --- | --- |
| `Tab` or `1`..`5` | change page: screen, palette, keymap, too small, footprint |
| `<-` `->` | change layout variant (screen page only) |
| `g` | switch the snapshot between Solved and Generating |
| `a` | switch the glyph set between Unicode and `--ascii` |
| `q` | quit |

To read the mock without a terminal:

```
cargo run -- capture plain    # layout only, no colour
cargo run -- capture ansi     # colour
```

Capture renders every view at 120 x 30 and at 80 x 24 through `TestBackend`.

## The three layouts

The variants disagree about where the chrome goes, because that is what
decides how much maze fits.

| | layout | chrome | 120 x 30 | 80 x 24 |
| --- | --- | --- | --- | --- |
| A | maze left, tall sidebar right | 36 columns, 3 rows | 20 x 13 cells | 10 x 10 cells |
| B | full-width bands stacked | 0 columns, 10 rows | 29 x 9 cells | 19 x 6 cells |
| C | maze maximal, two-row HUD | 0 columns, 2 rows | 29 x 13 cells | 19 x 10 cells |

The footprint page computes this table from the same constants the layouts
use, so the numbers cannot drift from the code.

## The hard-coded state

Seed 445, an 11 x 6 maze carved by Recursive Backtracker. Two snapshots:

- **Solved**: A\* has just reached the goal. 52 expanded, 5 still in the
  frontier, a path of 34. The previous run on the same maze was BFS: 64
  expanded, the same path of 34. The current cell sits on the path on
  purpose, so the priority order of decision 17 is visible.
- **Generating**: 45 steps in. 29 of 66 cells carved, 12 on the stack.

Both snapshots were produced offline and pasted in as literals. No code here
carves a maze or searches one.
