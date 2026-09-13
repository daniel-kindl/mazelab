//! The maze pane: it draws the display grid with the selected glyph set.
//!
//! One display-grid position takes **two screen columns**, so a maze occupies
//! `4W + 2` columns by `2H + 1` rows, and the maze is centred in the pane on
//! both axes. ADR 0007.

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::{Activity, App};
use crate::run::ready_state;
use crate::ui::palette::GlyphSet;
use crate::ui::shown_activity;

/// Draws the maze of `app`, centred in `area`.
///
/// Section 7.3 gives one cell-state function for each kind of run. The phase
/// decides which function draws a position:
///
/// - A generation run, animating or paused, draws with
///   [`crate::run::Run::gen_state`].
/// - A solver run, animating, paused or solved, draws with
///   [`crate::run::Run::solver_state`].
/// - Every other phase draws the maze with no run, with [`ready_state`].
///
/// A maze that does not fit the pane is not drawn. The maze-fit panel of
/// section 13 is what the pane shows then.
pub fn render(app: &App, glyphs: &GlyphSet, area: Rect, buffer: &mut Buffer) {
    let maze = &app.maze;
    let columns = maze.display_width().saturating_mul(2);
    let rows = maze.display_height();
    if columns > area.width || rows > area.height {
        return;
    }
    let left = area.x + (area.width - columns) / 2;
    let top = area.y + (area.height - rows) / 2;

    let activity = shown_activity(app.phase);
    for dy in 0..rows {
        for dx in 0..maze.display_width() {
            let (glyph, style) = match (activity, app.run.as_ref()) {
                (Some(Activity::Generating), Some(run)) => {
                    glyphs.gen_swatch(run.gen_state(maze, dx, dy))
                }
                (Some(Activity::Solving), Some(run)) => {
                    glyphs.solver_swatch(run.solver_state(maze, dx, dy))
                }
                _ => glyphs.solver_swatch(ready_state(maze, dx, dy)),
            };
            buffer.set_string(left + 2 * dx, top + dy, glyph, style);
        }
    }
}
