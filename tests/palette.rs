//! Items 3, 4 and 5 of tier 3 of the test plan: the three invariants of the
//! palette of section 7.4.
//!
//! The three invariant tests turn the cross-platform rendering claim into a
//! proof. They prove that **MazeLab stays legible when colour is absent, or
//! when a theme renders two of its colours alike.** A later edit can break each
//! rule without a visible sign, so each rule is a test and not only prose.
//! ADR 0007.
//!
//! Section 11.3 says "in one phase". A glyph set has one table of glyphs for
//! solver cell states and one for generation cell states, and every phase draws
//! from one of the two tables. These tests hold each table on its own.

use mazelab::run::{GenCellState, SolverCellState};
use mazelab::ui::palette::{self, GlyphSet};
use ratatui::backend::IntoCrossterm;
use ratatui::crossterm::style::Color as CrosstermColor;
use ratatui::style::Style;

/// Every solver cell state, in variant order.
///
/// Two checks hold this list to the enum, at compile time. The match does not
/// compile when a variant is added. The loop fails the build when an entry is
/// not the variant at its index, so no entry can be missing, repeated or out of
/// order.
const SOLVER_STATES: [SolverCellState; 8] = {
    use SolverCellState::{Current, Expanded, Frontier, Goal, Open, Path, Start, Wall};
    const fn exhaustive(state: SolverCellState) {
        match state {
            Start | Goal | Current | Path | Frontier | Expanded | Open | Wall => {}
        }
    }
    let states = [Start, Goal, Current, Path, Frontier, Expanded, Open, Wall];
    let mut index = 0;
    while index < states.len() {
        exhaustive(states[index]);
        assert!(states[index] as usize == index);
        index += 1;
    }
    states
};

/// Every generation cell state, in variant order.
///
/// The same two checks as [`SOLVER_STATES`] hold this list to the enum.
const GEN_STATES: [GenCellState; 4] = {
    use GenCellState::{Carved, Current, Frontier, Uncarved};
    const fn exhaustive(state: GenCellState) {
        match state {
            Current | Frontier | Carved | Uncarved => {}
        }
    }
    let states = [Current, Frontier, Carved, Uncarved];
    let mut index = 0;
    while index < states.len() {
        exhaustive(states[index]);
        assert!(states[index] as usize == index);
        index += 1;
    }
    states
};

/// The two glyph sets, with the name a failure message gives each.
const GLYPH_SETS: [(&str, GlyphSet); 2] =
    [("Unicode", palette::UNICODE), ("ASCII", palette::ASCII)];

/// One drawn cell state: its name, its glyph and its style.
struct Swatch {
    state: String,
    glyph: &'static str,
    style: Style,
}

/// The two tables of one glyph set, each with its name: first the solver cell
/// states, then the generation cell states.
fn tables(glyphs: &GlyphSet) -> [(&'static str, Vec<Swatch>); 2] {
    let solver = SOLVER_STATES
        .iter()
        .map(|&state| {
            let (glyph, style) = glyphs.solver_swatch(state);
            Swatch {
                state: format!("{state:?}"),
                glyph,
                style,
            }
        })
        .collect();
    let generation = GEN_STATES
        .iter()
        .map(|&state| {
            let (glyph, style) = glyphs.gen_swatch(state);
            Swatch {
                state: format!("{state:?}"),
                glyph,
                style,
            }
        })
        .collect();
    [("solver", solver), ("generation", generation)]
}

// ---------------------------------------------------------------------------
// Item 3 of section 11.3
// ---------------------------------------------------------------------------

/// Rule 1 of section 7.4: the glyph alone identifies the cell state.
#[test]
fn no_two_cell_states_in_one_phase_share_a_glyph_in_either_glyph_set() {
    for (set, glyphs) in GLYPH_SETS {
        for (table, swatches) in tables(&glyphs) {
            for (index, swatch) in swatches.iter().enumerate() {
                if let Some(other) = swatches[..index]
                    .iter()
                    .find(|other| other.glyph == swatch.glyph)
                {
                    panic!(
                        "in the {set} glyph set, the {table} cell states {} and {} \
                         share the glyph {:?}",
                        other.state, swatch.state, swatch.glyph
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Item 4 of section 11.3
// ---------------------------------------------------------------------------

/// `--ascii` is the guarantee of section 7.4, so it holds only if no glyph of
/// the ASCII glyph set leaves the ASCII range.
#[test]
fn every_glyph_in_the_ascii_glyph_set_is_below_u_0080() {
    for (table, swatches) in tables(&palette::ASCII) {
        for swatch in swatches {
            for character in swatch.glyph.chars() {
                assert!(
                    u32::from(character) < 0x80,
                    "the ASCII glyph {:?} of the {table} cell state {} holds U+{:04X}",
                    swatch.glyph,
                    swatch.state,
                    u32::from(character)
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Item 5 of section 11.3
// ---------------------------------------------------------------------------

/// Every colour one style sets, converted to the colour crossterm writes.
fn crossterm_colours(style: Style) -> Vec<CrosstermColor> {
    [style.fg, style.bg, style.underline_color]
        .into_iter()
        .flatten()
        .map(IntoCrossterm::into_crossterm)
        .collect()
}

/// Rule 4 of section 7.4: `Yellow` is banished from the maze palette.
///
/// The test reads each style through the swatch that the maze pane draws with,
/// in both glyph sets. It converts each colour through the crossterm backend,
/// so it holds the colour that reaches the terminal. That colour is
/// `DarkYellow`, which renders grey in PowerShell. Index 3 of the 256-colour
/// table is the same terminal colour, so the test refuses it too.
#[test]
fn dark_yellow_does_not_appear_in_the_maze_palette() {
    let banished = [CrosstermColor::DarkYellow, CrosstermColor::AnsiValue(3)];
    for (set, glyphs) in GLYPH_SETS {
        for (table, swatches) in tables(&glyphs) {
            for swatch in swatches {
                for colour in crossterm_colours(swatch.style) {
                    assert!(
                        !banished.contains(&colour),
                        "in the {set} glyph set, the style of the {table} cell state {} \
                         writes {colour:?}",
                        swatch.state
                    );
                }
            }
        }
    }
}

/// The names do not match across the two crates, so this test pins them:
/// ratatui's `Yellow` is the colour that crossterm writes as `DarkYellow`, and
/// the helper above sees it.
#[test]
fn ratatui_yellow_is_crossterm_dark_yellow() {
    let style = Style::new().fg(ratatui::style::Color::Yellow);
    assert_eq!(
        crossterm_colours(style),
        [CrosstermColor::DarkYellow],
        "the colour that the banishment test refuses"
    );
}
