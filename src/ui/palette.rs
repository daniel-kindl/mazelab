//! The palette: the map from a cell state to a colour and a glyph. The map is
//! a flat table with no logic in it.
//!
//! **The priority is not here.** A cell resolves to one cell state in
//! [`crate::run`], and this table only says how that state is drawn. That is
//! why `--ascii` costs nothing: the same cell state, a different glyph set.
//! Section 7.3.
//!
//! Every table below is indexed by the variant order of its cell state, which
//! is the priority order of section 7.3. Section 7.4 holds the four rules the
//! tables obey:
//!
//! 1. **The glyph alone identifies the cell state; colour only reinforces it.**
//!    The 16 named colours carry no contrast guarantee, because the user
//!    controls what each one renders as.
//! 2. **The shade ramp is monotone and carries the ranking**: `██` wall, `▓▓`
//!    path, `▒▒` frontier, `░░` expanded, blank floor.
//! 3. **Start, goal and current are letters in both glyph sets.**
//! 4. **`Yellow` (crossterm's `DarkYellow`) is banished from the maze
//!    palette.** It renders grey in PowerShell. ADR 0007.
//!
//! The light colours are named directly. `\x1b[1m` is not a reliable route to
//! a bright colour, because Windows Terminal makes `intenseTextStyle`
//! user-settable. Bold is used only where section 7.4 asks for it, and no
//! colour here depends on it.

use ratatui::style::{Color, Modifier, Style};

use crate::run::{GenCellState, SolverCellState};

/// The complete mapping from cell state to the characters that draw it.
///
/// **A glyph set is selected as a whole, and the two are never mixed.** A
/// caller picks [`UNICODE`] or [`ASCII`] once for a frame and reads every glyph
/// from that one value, so no frame can draw a glyph from each.
///
/// Every glyph is two characters wide, because a display-grid position takes
/// two screen columns (ADR 0007).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct GlyphSet {
    /// One glyph for each [`SolverCellState`], in variant order.
    pub solver: [&'static str; 8],
    /// One glyph for each [`GenCellState`], in variant order.
    pub generation: [&'static str; 4],
}

impl GlyphSet {
    /// The glyph that draws a solver cell state.
    #[must_use]
    pub const fn solver_glyph(&self, state: SolverCellState) -> &'static str {
        self.solver[state as usize]
    }

    /// The glyph that draws a generation cell state.
    ///
    /// The field is `generation` and not `gen`, because `gen` is a reserved
    /// keyword from edition 2024.
    #[must_use]
    pub const fn gen_glyph(&self, state: GenCellState) -> &'static str {
        self.generation[state as usize]
    }

    /// The glyph and the style that draw a solver cell state.
    #[must_use]
    pub const fn solver_swatch(&self, state: SolverCellState) -> (&'static str, Style) {
        (self.solver_glyph(state), SOLVER_STYLES[state as usize])
    }

    /// The glyph and the style that draw a generation cell state.
    #[must_use]
    pub const fn gen_swatch(&self, state: GenCellState) -> (&'static str, Style) {
        (self.gen_glyph(state), GEN_STYLES[state as usize])
    }
}

/// The glyph set that uses block glyphs.
///
/// Only `U+2588` is verified against the Windows Terminal default face.
/// `U+2591`, `U+2592` and `U+2593` sit in the same Block Elements range and are
/// assumed to follow. [`ASCII`] is the guarantee, not the assumption.
pub const UNICODE: GlyphSet = GlyphSet {
    //       Start Goal  Current Path  Frontier Expanded Open  Wall
    solver: ["SS", "GG", "@@", "▓▓", "▒▒", "░░", "  ", "██"],
    //           Current Frontier Carved Uncarved
    generation: ["@@", "▒▒", "  ", "██"],
};

/// The glyph set that uses ASCII characters only. `--ascii` selects it.
pub const ASCII: GlyphSet = GlyphSet {
    //       Start Goal  Current Path  Frontier Expanded Open  Wall
    solver: ["SS", "GG", "@@", "**", "::", "..", "  ", "##"],
    //           Current Frontier Carved Uncarved
    generation: ["@@", "::", "  ", "##"],
};

/// The style of each [`SolverCellState`], in variant order. The same in both
/// glyph sets.
pub const SOLVER_STYLES: [Style; 8] = [
    // Start
    Style::new()
        .fg(Color::LightGreen)
        .add_modifier(Modifier::BOLD),
    // Goal
    Style::new()
        .fg(Color::LightRed)
        .add_modifier(Modifier::BOLD),
    // Current
    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
    // Path
    Style::new().fg(Color::Green),
    // Frontier
    Style::new().fg(Color::LightCyan),
    // Expanded
    Style::new().fg(Color::Blue),
    // Open floor: the terminal default.
    Style::new(),
    // Wall
    Style::new().fg(Color::DarkGray),
];

/// The style of each [`GenCellState`], in variant order. The same in both
/// glyph sets.
pub const GEN_STYLES: [Style; 4] = [
    // Current
    Style::new().fg(Color::White).add_modifier(Modifier::BOLD),
    // Frontier
    Style::new().fg(Color::LightMagenta),
    // Carved: the terminal default.
    Style::new(),
    // Uncarved
    Style::new().fg(Color::DarkGray),
];
