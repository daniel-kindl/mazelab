//! The rendering. This is the only part of the library that names the terminal.

pub mod help;
pub mod layout;
pub mod legend;
pub mod maze;
pub mod palette;
pub mod panel;
pub mod stats;

use ratatui::Frame;
use ratatui::text::Line;

use crate::app::App;

/// Draws one frame of the screen.
///
/// **This is a placeholder and it is not the screen of section 7.** The bands,
/// the palette, the glyph sets, the statistics band, the legend row, the help
/// row and the two too-small panels are the rendering issues, and each of them
/// replaces part of this function. It draws one line so that the loop of
/// section 6 has the call it is specified to make, and so that a run can be
/// watched while the screen is written.
pub fn render(frame: &mut Frame, app: &App) {
    let steps = app.run.as_ref().map_or(0, |run| run.stats(&app.maze).steps);
    let line = Line::from(format!(
        "MazeLab  seed {}  maze {}x{}  phase {:?}  steps {steps}  \
         [placeholder screen: g generate, s solve, Space pause, . step, q quit]",
        app.seed,
        app.maze.width(),
        app.maze.height(),
        app.phase,
    ));
    let area = frame.area();
    frame.render_widget(line, area);
}
