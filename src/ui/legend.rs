//! The legend row: it names each cell state that is on the screen.
//!
//! **The legend is the in-application evidence** that the glyphs and the
//! colours arrived intact. No test can render a glyph on a real font, so the
//! legend is part of the decision of ADR 0007, and it is more than chrome.
//! Its contract:
//!
//! **It shows every cell state of the current phase, in the live glyph set and
//! the live colour, and it never drops an entry.**
//!
//! Two forms, on the breakpoint of [`BREAKPOINT`] columns. Section 7.6.
//!
//! | form | width | labels | separator |
//! | --- | --- | --- | --- |
//! | full, at 100 columns and above | 85 | `start goal current path frontier expanded floor wall` | two spaces |
//! | short, below 100 columns | 74 | `start goal cur path frontier expanded floor wall` | one space |
//!
//! The widths are those of the solver legend. The generation legend has four
//! entries and is shorter. `frontier` and `expanded` are glossary terms, so no
//! form shortens them. The swatch of a state whose glyph is blank is
//! bracketed, `[  ]`, because a blank swatch shows nothing.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Widget;

use crate::app::{Activity, App};
use crate::ui::layout::BREAKPOINT;
use crate::ui::palette::{GEN_STYLES, GlyphSet, SOLVER_STYLES};
use crate::ui::shown_activity;

/// The two labels of one cell state: the full one, and the one below the
/// breakpoint.
#[derive(Clone, Copy)]
struct Label {
    /// The label at [`BREAKPOINT`] columns and above.
    full: &'static str,
    /// The label below [`BREAKPOINT`] columns.
    short: &'static str,
}

/// A label that is the same in both forms.
const fn same(label: &'static str) -> Label {
    Label {
        full: label,
        short: label,
    }
}

/// The label of each solver cell state, in variant order.
const SOLVER_LABELS: [Label; 8] = [
    same("start"),
    same("goal"),
    Label {
        full: "current",
        short: "cur",
    },
    same("path"),
    same("frontier"),
    same("expanded"),
    same("floor"),
    same("wall"),
];

/// The label of each generation cell state, in variant order.
const GEN_LABELS: [Label; 4] = [
    Label {
        full: "current",
        short: "cur",
    },
    same("frontier"),
    same("carved"),
    same("uncarved"),
];

/// Draws the legend row of `app` in `area`, in the glyph set of the frame.
///
/// A phase that draws a generation run shows the generation legend. Every
/// other phase draws with the solver cell states, so it shows the solver
/// legend.
pub fn render(app: &App, glyphs: &GlyphSet, area: Rect, buffer: &mut Buffer) {
    let activity = shown_activity(app.phase).unwrap_or(Activity::Solving);
    line(activity, glyphs, area.width).render(area, buffer);
}

/// The legend of one kind of run, in the form that fits `width` columns.
///
/// Each entry is the swatch, in the glyph and the style that draw its cell
/// state in the maze pane, then a space and the label. The glyph set is the
/// one the caller selected for the frame, so the legend draws the same glyphs
/// as the maze.
#[must_use]
pub fn line(activity: Activity, glyphs: &GlyphSet, width: u16) -> Line<'static> {
    let entries: Vec<_> = match activity {
        Activity::Generating => glyphs
            .generation
            .into_iter()
            .zip(GEN_STYLES)
            .zip(GEN_LABELS)
            .collect(),
        Activity::Solving => glyphs
            .solver
            .into_iter()
            .zip(SOLVER_STYLES)
            .zip(SOLVER_LABELS)
            .collect(),
    };
    let full = width >= BREAKPOINT;
    let separator = if full { "  " } else { " " };
    let mut spans = Vec::new();
    for ((glyph, style), label) in entries {
        if !spans.is_empty() {
            spans.push(Span::raw(separator));
        }
        if glyph.trim().is_empty() {
            spans.push(Span::raw("["));
            spans.push(Span::styled(glyph, style));
            spans.push(Span::raw("]"));
        } else {
            spans.push(Span::styled(glyph, style));
        }
        let label = if full { label.full } else { label.short };
        spans.push(Span::raw(format!(" {label}")));
    }
    Line::from(spans)
}
