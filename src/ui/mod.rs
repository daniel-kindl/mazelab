//! The rendering. This is the only part of the library that names the terminal.

pub mod help;
pub mod layout;
pub mod legend;
pub mod maze;
pub mod palette;
pub mod panel;
pub mod stats;

use ratatui::Frame;

use crate::app::{Activity, App, Phase};

/// Draws one frame of the screen of section 7.
///
/// The layout is the five bands of section 7.1, and it is the same at every
/// terminal size. The glyph set is selected here, once for the frame, and
/// every band that draws a glyph reads it from this one value, so no frame can
/// mix the two glyph sets.
///
/// The legend row, the help row and the two too-small panels of section 13
/// are not drawn yet. Their bands stay blank, and a maze that does not fit the
/// pane is not drawn.
pub fn render(frame: &mut Frame, app: &App) {
    let glyphs = if app.ascii {
        &palette::ASCII
    } else {
        &palette::UNICODE
    };
    let screen = frame.area();
    let bands = layout::bands(screen);
    frame.render_widget(layout::status_line(app), bands.status);
    maze::render(app, glyphs, bands.maze, frame.buffer_mut());
    stats::render(app, bands.stats, frame.buffer_mut());
    if app.help_open {
        help::render_overlay(screen, bands.maze, frame.buffer_mut());
    }
}

/// The kind of run that the phase draws on the maze and in the band, or `None`
/// where the phase draws no run in progress.
///
/// It is [`Phase::activity`], with one addition: `Solved` holds a solver run
/// and has no activity, because nothing is left to advance, but the screen
/// still draws that run. `Ready` answers `None`. The generation run it can
/// hold is over.
const fn shown_activity(phase: Phase) -> Option<Activity> {
    match phase {
        Phase::Solved => Some(Activity::Solving),
        phase => phase.activity(),
    }
}
