//! Candidate palette and glyph set. THROWAWAY.
//!
//! Decision 16 limits the palette to the 16 ANSI named colours, so the maze
//! inherits the user's terminal theme. Issue #4 found the price: named colours
//! carry no contrast guarantee, so shape has to carry meaning as well as
//! colour. Every state below is therefore distinct in glyph *and* in colour.
//!
//! Only U+2588 was verified against the Windows Terminal default face. The
//! other three glyphs used here (U+2591..U+2593) sit in the same Block
//! Elements range, and `--ascii` is the escape hatch either way.

use ratatui::style::{Color, Modifier, Style};

use crate::mock::{Cell, GenCell};

pub struct Look {
    pub glyph: &'static str,
    pub color: Color,
    pub bold: bool,
}

impl Look {
    pub fn style(&self) -> Style {
        let s = Style::new().fg(self.color);
        if self.bold { s.add_modifier(Modifier::BOLD) } else { s }
    }
}

const fn look(glyph: &'static str, color: Color, bold: bool) -> Look {
    Look { glyph, color, bold }
}

pub fn solver_look(c: Cell, ascii: bool) -> Look {
    match (c, ascii) {
        (Cell::Start, false) => look("SS", Color::LightGreen, true),
        (Cell::Start, true) => look("SS", Color::LightGreen, true),
        (Cell::Goal, false) => look("GG", Color::LightRed, true),
        (Cell::Goal, true) => look("GG", Color::LightRed, true),
        (Cell::Current, false) => look("@@", Color::White, true),
        (Cell::Current, true) => look("@@", Color::White, true),
        (Cell::Path, false) => look("▓▓", Color::Green, true),
        (Cell::Path, true) => look("**", Color::Green, true),
        (Cell::Frontier, false) => look("▒▒", Color::LightCyan, false),
        (Cell::Frontier, true) => look("::", Color::LightCyan, false),
        (Cell::Expanded, false) => look("░░", Color::Blue, false),
        (Cell::Expanded, true) => look("..", Color::Blue, false),
        (Cell::Floor, _) => look("  ", Color::Reset, false),
        (Cell::Wall, false) => look("██", Color::DarkGray, false),
        (Cell::Wall, true) => look("##", Color::DarkGray, false),
    }
}

pub fn gen_look(c: GenCell, ascii: bool) -> Look {
    match (c, ascii) {
        (GenCell::Current, _) => look("@@", Color::White, true),
        (GenCell::Frontier, false) => look("▒▒", Color::LightMagenta, false),
        (GenCell::Frontier, true) => look("::", Color::LightMagenta, false),
        (GenCell::Carved, _) => look("  ", Color::Reset, false),
        (GenCell::Uncarved, false) => look("██", Color::DarkGray, false),
        (GenCell::Uncarved, true) => look("##", Color::DarkGray, false),
    }
}

pub const SOLVER_STATES: &[(Cell, &str)] = &[
    (Cell::Start, "start"),
    (Cell::Goal, "goal"),
    (Cell::Current, "current"),
    (Cell::Path, "final path"),
    (Cell::Frontier, "frontier"),
    (Cell::Expanded, "expanded"),
    (Cell::Floor, "open floor"),
    (Cell::Wall, "wall"),
];

pub const GEN_STATES: &[(GenCell, &str)] = &[
    (GenCell::Current, "current"),
    (GenCell::Frontier, "frontier (stack)"),
    (GenCell::Carved, "carved"),
    (GenCell::Uncarved, "uncarved"),
];

pub const ANSI_16: &[(Color, &str)] = &[
    (Color::Black, "Black"),
    (Color::Red, "Red"),
    (Color::Green, "Green"),
    (Color::Yellow, "Yellow"),
    (Color::Blue, "Blue"),
    (Color::Magenta, "Magenta"),
    (Color::Cyan, "Cyan"),
    (Color::Gray, "Gray"),
    (Color::DarkGray, "DarkGray"),
    (Color::LightRed, "LightRed"),
    (Color::LightGreen, "LightGreen"),
    (Color::LightYellow, "LightYellow"),
    (Color::LightBlue, "LightBlue"),
    (Color::LightMagenta, "LightMagenta"),
    (Color::LightCyan, "LightCyan"),
    (Color::White, "White"),
];

pub const ANSI_16_SHORT: &[&str] = &[
    "Blk", "Red", "Grn", "Yel", "Blu", "Mag", "Cyn", "Gry",
    "DkGry", "LtRed", "LtGrn", "LtYel", "LtBlu", "LtMag", "LtCyn", "Wht",
];

pub const SOLVER_SHORT: &[&str] = &[
    "start", "goal", "cur", "path", "front", "exp", "floor", "wall",
];

pub const GEN_SHORT: &[&str] = &["cur", "stack", "carved", "uncarved"];
