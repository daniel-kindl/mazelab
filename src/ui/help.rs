//! The help row and the help overlay: they name the keys.
//!
//! The help row names the keys in the form that fits the breakpoint. The help
//! overlay holds the whole keymap of section 7.5, and `?` opens it over the
//! maze pane.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, Padding, Paragraph, Widget};

/// One row of the keymap: the keys, what they do, and a note.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyRow {
    /// The key or keys, as the user reads them on the keyboard.
    pub keys: &'static str,
    /// What the keys do.
    pub action: &'static str,
    /// What the user must know besides the action.
    pub note: &'static str,
}

/// The keymap of section 7.5, in the order of its table.
///
/// `main.rs` is where a key code becomes an action, and this table is what the
/// user reads about it. Three notes are shorter than the table in section 7.5,
/// and the `b` action names the cycle, so that the overlay is 76 columns wide
/// and fits the 79-column layout floor. The meaning of each row is unchanged.
pub const KEYMAP: [KeyRow; 16] = [
    key("g", "generate", "animated, from the current seed"),
    key("G", "generate instantly", "skips the animation"),
    key("s", "solve", "runs the selected solver"),
    key("Space", "pause / resume", "Generating and Solving"),
    key(".", "single step", "exactly one step at any speed; pauses"),
    key("+ -", "speed up / down", "one rung of the ladder"),
    key("1 2", "choose generator", "selects; does not generate"),
    key("3 4 5", "choose solver", "Solved becomes Ready"),
    key("Tab", "next solver", "shortcut for 3-5, wraps"),
    key("<- ->", "maze narrower / wider", "regenerates instantly"),
    key("Up Down", "maze taller / shorter", "regenerates instantly"),
    key(
        "f",
        "refit and regenerate",
        "fits the maze to the terminal now",
    ),
    key("n", "new seed", "fresh random seed; regenerates instantly"),
    key(
        "b",
        "cycle braid factor",
        "0.00, 0.25, 0.50; regenerates instantly",
    ),
    key("?", "help", "the full keymap over the maze pane"),
    key("q Esc", "quit", "restores the terminal"),
];

/// One row of [`KEYMAP`].
const fn key(keys: &'static str, action: &'static str, note: &'static str) -> KeyRow {
    KeyRow { keys, action, note }
}

/// The width of the widest value of one field of [`KEYMAP`].
fn widest(field: impl Fn(&KeyRow) -> &'static str) -> usize {
    KEYMAP.iter().map(|row| field(row).len()).max().unwrap_or(0)
}

/// The columns between two fields of an overlay row.
const GAP: usize = 2;

/// Draws the help overlay over the maze pane.
///
/// The box is sized to the keymap, and centred over `pane`. **It is never
/// clipped.** The keymap is 18 rows with its border, and the pane is shorter
/// than that on a small terminal: 14 rows at 80 x 24 and 9 at the layout
/// floor. The box then starts at the top of the pane and covers the chrome
/// below it as far as it must. It never covers the status row, and at the
/// layout floor of 19 rows the 18-row box still fits under it.
///
/// The overlay changes no phase, so an animation continues behind it. While it
/// is open, `Esc` closes it instead of quitting. Section 7.5.
pub fn render_overlay(screen: Rect, pane: Rect, buffer: &mut Buffer) {
    let keys = widest(|row| row.keys);
    let action = widest(|row| row.action);
    let note = widest(|row| row.note);
    let lines: Vec<Line> = KEYMAP
        .iter()
        .map(|row| {
            Line::from(format!(
                "{:<keys$}{:GAP$}{:<action$}{:GAP$}{}",
                row.keys, "", row.action, "", row.note
            ))
        })
        .collect();

    // One column of padding on each side and the border around it.
    let width = u16::try_from(keys + GAP + action + GAP + note + 4).unwrap_or(u16::MAX);
    let height = u16::try_from(KEYMAP.len() + 2).unwrap_or(u16::MAX);
    // The box never starts above the pane, so the status row stays whole:
    // section 7.2 keeps the seed on screen at all times.
    let centred = (pane.y + pane.height / 2).saturating_sub(height / 2);
    let area = Rect::new(
        pane.x + pane.width.saturating_sub(width) / 2,
        centred.max(pane.y),
        width,
        height,
    )
    .clamp(screen);

    let block = Block::bordered()
        .title(" keymap ")
        .title_bottom(Line::from(" ? or Esc closes ").right_aligned())
        .padding(Padding::horizontal(1));
    Clear.render(area, buffer);
    Paragraph::new(lines).block(block).render(area, buffer);
}
